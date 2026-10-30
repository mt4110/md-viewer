mod common;

use common::Project;
use serde_json::{Value, json};
use veil_boundary_lab::engine::{ContentCheck, DenialCode};
use veil_boundary_lab::report::ExpectationCheck;
use veil_boundary_lab::{evaluate_suite, load_suite};

const ALLOWED: &str = "cases/allowed/case.json";
const FOREIGN: &str = "cases/foreign-tenant/case.json";

fn load_error(project: &Project) -> veil_boundary_lab::LabError {
    load_suite(&project.fixtures())
        .err()
        .expect("must reject invalid fixture")
}

#[test]
fn five_real_results_keep_order_original_inputs_and_actual_utf8_costs() {
    let project = Project::new();
    let report = evaluate_suite(load_suite(&project.fixtures()).unwrap());
    assert_eq!(report.summary.total, 5);
    assert_eq!(report.summary.matched, 5);
    assert_eq!(report.summary.mismatched, 0);
    let expected = [
        ("allowed", "passed"),
        ("foreign-tenant", "tenant_mismatch"),
        ("unauthorized-id", "object_unauthorized"),
        ("extra-field", "unapproved_field"),
        ("value-not-classified", "passed"),
    ];
    for (case, (id, reason)) in report.cases.iter().zip(expected) {
        assert_eq!(case.input.id, id);
        assert_eq!(case.actual.code(), reason);
        assert_eq!(case.expectation_check, ExpectationCheck::Matched);
        assert_eq!(
            case.input.response.as_bytes(),
            std::fs::read(project.fixtures().join(format!("cases/{id}/response.json"))).unwrap()
        );
        assert_eq!(
            case.input.policy.as_bytes(),
            std::fs::read(project.fixtures().join("policy.json")).unwrap()
        );
        assert_eq!(
            case.input.context.as_bytes(),
            std::fs::read(project.fixtures().join("context.json")).unwrap()
        );
        if let ContentCheck::Passed { cost } = case.actual {
            assert_eq!(cost.records, 1);
            assert_eq!(cost.bytes, case.input.response.len() as u64);
        }
    }
    let first = &report.cases[0].input.response;
    assert!(first.len() > first.chars().count());
    assert!(first.ends_with('\n'));
    let mixed: Value = serde_json::from_str(&report.cases[1].input.response).unwrap();
    assert_eq!(mixed.as_array().unwrap().len(), 2);
    assert_eq!(mixed[0]["tenant_id"], "synthetic-acme");
    assert_eq!(mixed[1]["tenant_id"], "synthetic-other");
    let context: Value = serde_json::from_str(&report.cases[1].input.context).unwrap();
    assert!(
        context["record_ids"]
            .as_array()
            .unwrap()
            .contains(&mixed[1]["id"])
    );
}

#[test]
fn wrong_outcome_and_wrong_denial_reason_are_both_mismatches() {
    let project = Project::new();
    project.edit(ALLOWED, |v| {
        v["expected"] = json!({"content_check":"denied","reason":"tenant_mismatch"})
    });
    project.edit(FOREIGN, |v| {
        v["expected"]["reason"] = "unapproved_field".into()
    });
    let report = evaluate_suite(load_suite(&project.fixtures()).unwrap());
    assert_eq!(report.summary.mismatched, 2);
    for case in &report.cases[..2] {
        assert_eq!(case.expectation_check, ExpectationCheck::Mismatched);
        assert!(!case.highlights_enabled);
    }
    assert_eq!(report.cases[1].actual.code(), "tenant_mismatch");
}

#[test]
fn original_duplicate_keys_reach_core_without_normalization() {
    let project = Project::new();
    let body = b" {\"id\":\"1\",\"\\u0069d\":\"2\",\"tenant_id\":\"synthetic-acme\"}\n\n";
    project.write("cases/allowed/response.json", body);
    project.edit(ALLOWED, |v| {
        v["expected"] = json!({"content_check":"denied","reason":"invalid_json"})
    });
    let report = evaluate_suite(load_suite(&project.fixtures()).unwrap());
    assert_eq!(report.cases[0].input.response.as_bytes(), body);
    assert_eq!(report.cases[0].actual.code(), "invalid_json");
    assert_eq!(report.summary.mismatched, 0);
}

#[test]
fn selected_contract_is_preserved_when_multiple_policies_are_present() {
    let project = Project::new();
    let original = std::fs::read_to_string(project.fixtures().join("policy.json")).unwrap();
    let mut second: Value = serde_json::from_str::<Value>(&original).unwrap()[0].clone();
    second["id"] = "another.customers".into();
    second["permission"] = "another:permission".into();
    // Append without changing the existing policy's authored line annotations.
    let combined = format!(
        "{},\n{}\n]\n",
        original.trim_end().strip_suffix(']').unwrap(),
        serde_json::to_string_pretty(&second).unwrap()
    );
    project.write("policy.json", combined);
    project.edit("suite.json", |v| v["contract"] = "another.customers".into());
    let report = evaluate_suite(load_suite(&project.fixtures()).unwrap());
    assert_eq!(report.suite.contract, "another.customers");
    assert!(
        report
            .cases
            .iter()
            .all(|c| c.actual.code() == "unauthorized")
    );
    assert_eq!(
        serde_json::to_value(report).unwrap()["suite"]["contract"],
        "another.customers"
    );
}

#[test]
fn metadata_schema_unknown_duplicate_fields_and_invalid_expectations_fail() {
    for edit in [
        json!({"content_check":"passed","reason":"tenant_mismatch"}),
        json!({"content_check":"denied"}),
        json!({"content_check":"denied","reason":"unknown"}),
        json!({"content_check":"passed","extra":true}),
    ] {
        let project = Project::new();
        project.edit(ALLOWED, |v| v["expected"] = edit);
        assert_eq!(load_error(&project).code, "fixture_json");
    }
    for (field, value, code) in [
        ("schema_version", json!(2), "case_schema"),
        ("synthetic", json!(false), "case_schema"),
        ("id", json!("bad/id"), "case_id"),
        ("unexpected", json!(true), "fixture_json"),
    ] {
        let project = Project::new();
        project.edit(ALLOWED, |v| v[field] = value);
        assert_eq!(load_error(&project).code, code);
    }
    let project = Project::new();
    let original = std::fs::read_to_string(project.fixtures().join(ALLOWED)).unwrap();
    project.write(
        ALLOWED,
        original.replacen(
            "\"schema_version\": 1",
            "\"schema_version\": 1, \"schema_version\": 1",
            1,
        ),
    );
    assert_eq!(load_error(&project).code, "fixture_json");
}

#[test]
fn suite_and_policy_errors_do_not_become_content_denials() {
    for (field, value, code) in [
        ("schema_version", json!(2), "suite_schema"),
        ("version", json!(0), "suite_schema"),
        ("cases", json!([]), "suite_cases"),
        (
            "cases",
            json!(vec!["cases/allowed/case.json"; 17]),
            "suite_cases",
        ),
        (
            "cases",
            json!(["cases/allowed/case.json", "cases/allowed/case.json"]),
            "duplicate_case",
        ),
        ("contract", json!("not-configured"), "contract"),
        ("unexpected", json!(true), "fixture_json"),
    ] {
        let project = Project::new();
        project.edit("suite.json", |v| v[field] = value);
        assert_eq!(load_error(&project).code, code);
    }
    let project = Project::new();
    project.edit("policy.json", |v| v[0]["max_records"] = json!(0));
    assert_eq!(load_error(&project).code, "configuration");
    let project = Project::new();
    project.write("context.json", "{\"principal\":\"SYNTHETIC_PARSE_MARKER");
    let error = load_error(&project);
    assert_eq!(error.code, "fixture_json");
    assert!(!error.to_string().contains("SYNTHETIC_PARSE_MARKER"));
    let project = Project::new();
    let original = std::fs::read_to_string(project.fixtures().join("context.json")).unwrap();
    project.write(
        "context.json",
        original.replace("customer:read", "another:permission"),
    );
    let report = evaluate_suite(load_suite(&project.fixtures()).unwrap());
    assert!(
        report
            .cases
            .iter()
            .all(|c| c.actual.code() == "unauthorized")
    );
}

#[test]
fn missing_nonregular_and_escaping_files_fail_before_inspection() {
    for (path, code) in [
        ("missing.json", "fixture_file"),
        ("cases", "fixture_file"),
        ("../policy.json", "fixture_path"),
        ("/tmp/boundary-outside.json", "fixture_path"),
    ] {
        let project = Project::new();
        project.edit(ALLOWED, |v| v["response_file"] = path.into());
        assert_eq!(load_error(&project).code, code);
    }
}

#[cfg(unix)]
#[test]
fn symlinks_to_files_directories_and_outside_root_are_rejected() {
    for (link, target, response) in [
        ("linked.json", "policy.json", "linked.json"),
        ("linked", "cases/allowed", "linked/response.json"),
        ("linked.json", "/etc/hosts", "linked.json"),
    ] {
        let project = Project::new();
        std::os::unix::fs::symlink(target, project.fixtures().join(link)).unwrap();
        project.edit(ALLOWED, |v| v["response_file"] = response.into());
        assert_eq!(load_error(&project).code, "fixture_path");
    }
}

#[test]
fn bounded_reads_distinguish_lab_limits_from_core_byte_denial() {
    let project = Project::new();
    project.write(
        "cases/allowed/response.json",
        vec![b'x'; leak_fence_core::MAX_BODY_BYTES + 1],
    );
    let report = evaluate_suite(load_suite(&project.fixtures()).unwrap());
    assert_eq!(
        report.cases[0].actual,
        ContentCheck::Denied {
            reason: DenialCode::ByteLimit
        }
    );
    project.write(
        "cases/allowed/response.json",
        vec![b'x'; leak_fence_core::MAX_BODY_BYTES + 2],
    );
    assert_eq!(load_error(&project).code, "fixture_limit");
    let project = Project::new();
    project.write(ALLOWED, vec![b' '; 65_537]);
    assert_eq!(load_error(&project).code, "fixture_limit");
    let project = Project::new();
    project.write("cases/allowed/response.json", [0xff]);
    assert_eq!(load_error(&project).code, "fixture_utf8");
}

#[test]
fn stale_out_of_range_overlapping_and_invalid_highlights_are_errors() {
    for (field, value, code) in [
        ("start_line", json!(0), "highlight_range"),
        ("end_line", json!(999), "highlight_range"),
        ("start_line", json!(4), "highlight_range"),
        (
            "expected_text",
            json!("stale synthetic text"),
            "highlight_text",
        ),
        ("id", json!("not/an/id"), "highlight_id"),
    ] {
        let project = Project::new();
        project.edit(FOREIGN, |v| v["highlights"][0][field] = value);
        assert_eq!(load_error(&project).code, code);
    }
    for (same_id, code) in [(true, "highlight_id"), (false, "highlight_overlap")] {
        let project = Project::new();
        project.edit(FOREIGN, |v| {
            let mut copy = v["highlights"][0].clone();
            if !same_id {
                copy["id"] = "another-id".into();
            }
            v["highlights"].as_array_mut().unwrap().push(copy);
        });
        assert_eq!(load_error(&project).code, code);
    }
    let project = Project::new();
    project.write("cases/foreign-tenant/response.json", "[]\n");
    assert_eq!(load_error(&project).code, "highlight_range");
}
