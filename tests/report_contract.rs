mod common;

use common::Project;
use serde_json::{Value, json};
use std::fs;
use std::process::{Command, Output};
use veil_boundary_lab::engine::ENGINE_REV;

impl Project {
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_veil-boundary-lab"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .unwrap()
    }
}

#[test]
fn check_is_read_only_and_expected_denials_exit_successfully() {
    let project = Project::new();
    let before: Vec<_> = fs::read_dir(&project.root)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    let output = project.run(&["check"]);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("total=5 matched=5 mismatched=0"));
    assert!(text.contains("result=tenant_mismatch expectation=matched"));
    assert!(!text.contains("synthetic-acme"));
    assert!(!text.contains("合成顧客"));
    assert!(output.stderr.is_empty());
    let after: Vec<_> = fs::read_dir(&project.root)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(before, after);
}

#[test]
fn build_is_deterministic_json_only_and_preserves_all_input_bytes() {
    let project = Project::new();
    for name in ["nested/first", "nested/second"] {
        let output = project.run(&["build", "--out", name]);
        assert_eq!(output.status.code(), Some(0), "{:?}", output.stderr);
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("stage=m2-json-html")
        );
        assert!(output.stderr.is_empty());
        assert!(project.root.join(name).join("index.html").exists());
    }
    assert_eq!(
        fs::read(project.root.join("nested/first/index.html")).unwrap(),
        fs::read(project.root.join("nested/second/index.html")).unwrap()
    );
    let first = fs::read(project.root.join("nested/first/report.json")).unwrap();
    assert_eq!(
        first,
        fs::read(project.root.join("nested/second/report.json")).unwrap()
    );
    let report: Value = serde_json::from_slice(&first).unwrap();
    assert_eq!(report["engine"]["git_rev"], ENGINE_REV);
    assert_eq!(report["suite"]["contract"], "sample.customers");
    assert_eq!(
        report["summary"],
        json!({"total":5,"matched":5,"mismatched":0})
    );
    assert_eq!(
        report["scope"],
        json!({"content_only":true,"budget_check":"not_performed","network":"not_used","data":"synthetic"})
    );
    for case in report["cases"].as_array().unwrap() {
        let input = &case["input"];
        let id = input["id"].as_str().unwrap();
        assert_eq!(
            input["response"].as_str().unwrap().as_bytes(),
            fs::read(project.fixtures().join(format!("cases/{id}/response.json"))).unwrap()
        );
        assert_eq!(
            input["policy"].as_str().unwrap().as_bytes(),
            fs::read(project.fixtures().join("policy.json")).unwrap()
        );
        assert_eq!(
            input["context"].as_str().unwrap().as_bytes(),
            fs::read(project.fixtures().join("context.json")).unwrap()
        );
        if case["actual"]["content_check"] == "denied" {
            assert!(case["actual"].get("cost").is_none());
        }
    }
    assert!(
        !String::from_utf8(first)
            .unwrap()
            .contains(project.root.to_str().unwrap())
    );
}

#[test]
fn mismatches_exit_two_and_keep_actual_and_expected_separate() {
    let project = Project::new();
    project.edit("cases/foreign-tenant/case.json", |v| {
        v["expected"]["reason"] = "unapproved_field".into()
    });
    assert_eq!(project.run(&["check"]).status.code(), Some(2));
    assert_eq!(
        project.run(&["build", "--out", "mismatch"]).status.code(),
        Some(2)
    );
    let report: Value =
        serde_json::from_slice(&fs::read(project.root.join("mismatch/report.json")).unwrap())
            .unwrap();
    let case = &report["cases"][1];
    assert_eq!(case["actual"]["reason"], "tenant_mismatch");
    assert_eq!(case["input"]["expected"]["reason"], "unapproved_field");
    assert_eq!(case["expectation_check"], "mismatched");
    assert_eq!(case["highlights_enabled"], false);
}

#[test]
fn existing_directories_files_and_sentinels_are_preserved() {
    let project = Project::new();
    fs::create_dir(project.root.join("existing")).unwrap();
    fs::write(project.root.join("existing/sentinel"), b"keep").unwrap();
    fs::write(project.root.join("file"), b"keep-file").unwrap();
    for name in ["existing", "file", "file/child"] {
        let output = project.run(&["build", "--out", name]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
    }
    assert_eq!(
        fs::read(project.root.join("existing/sentinel")).unwrap(),
        b"keep"
    );
    assert_eq!(fs::read(project.root.join("file")).unwrap(), b"keep-file");
    assert!(!project.root.join("existing/report.json").exists());
    assert_eq!(
        project.run(&["build", "--out", "fresh"]).status.code(),
        Some(0)
    );
    let before = fs::read(project.root.join("fresh/report.json")).unwrap();
    assert_eq!(
        project.run(&["build", "--out", "fresh"]).status.code(),
        Some(1)
    );
    assert_eq!(
        before,
        fs::read(project.root.join("fresh/report.json")).unwrap()
    );
}

#[cfg(unix)]
#[test]
fn dangling_output_symlinks_are_not_replaced() {
    let project = Project::new();
    std::os::unix::fs::symlink("missing-target", project.root.join("link")).unwrap();
    assert_eq!(
        project.run(&["build", "--out", "link"]).status.code(),
        Some(1)
    );
    assert_eq!(
        fs::read_link(project.root.join("link")).unwrap(),
        std::path::PathBuf::from("missing-target")
    );
}

#[test]
fn invalid_input_usage_and_output_failure_exit_one_without_input_dumps() {
    let project = Project::new();
    for args in [
        vec![],
        vec!["unknown"],
        vec!["build"],
        vec!["build", "--out"],
        vec!["check", "extra"],
        vec!["build", "--force", "new"],
    ] {
        assert_eq!(project.run(&args).status.code(), Some(1));
    }
    project.write("context.json", "{\"principal\":\"SYNTHETIC_PARSE_MARKER");
    let output = project.run(&["build", "--out", "never-created"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error=fixture_json\n"
    );
    assert!(!project.root.join("never-created").exists());
}

#[test]
fn help_works_without_fixtures_and_explains_the_intermediate_stage() {
    let output = Command::new(env!("CARGO_BIN_EXE_veil-boundary-lab"))
        .arg("--help")
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("M2"));
    assert!(text.contains("offline index.html"));
    assert!(text.contains("repository root"));
    let output = Command::new(env!("CARGO_BIN_EXE_veil-boundary-lab"))
        .arg("check")
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
}

#[cfg(unix)]
#[test]
fn closed_stdout_is_an_io_failure_even_when_all_cases_match() {
    use std::os::fd::OwnedFd;
    use std::os::unix::net::UnixStream;
    use std::process::Stdio;
    let project = Project::new();
    let (reader, writer) = UnixStream::pair().unwrap();
    drop(reader);
    let output = Command::new(env!("CARGO_BIN_EXE_veil-boundary-lab"))
        .arg("check")
        .current_dir(&project.root)
        .stdout(Stdio::from(OwnedFd::from(writer)))
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error=stdout_write\n"
    );
}

#[test]
fn resolved_engine_is_the_reported_git_revision_and_has_no_worker_dependency() {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--frozen", "--format-version", "1"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output.stderr);
    let metadata: Value = serde_json::from_slice(&output.stdout).unwrap();
    let packages = metadata["packages"].as_array().unwrap();
    let core = packages
        .iter()
        .find(|p| p["name"] == "leak-fence-core")
        .unwrap();
    assert!(
        core["source"]
            .as_str()
            .unwrap()
            .ends_with(&format!("#{ENGINE_REV}"))
    );
    assert!(
        core["source"]
            .as_str()
            .unwrap()
            .contains(&format!("?rev={ENGINE_REV}"))
    );
    assert!(
        !packages
            .iter()
            .any(|p| p["name"].as_str().unwrap().contains("worker"))
    );
    let lab = packages
        .iter()
        .find(|p| p["name"] == "veil-boundary-lab")
        .unwrap();
    assert!(
        lab["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .all(|dep| dep["path"].is_null())
    );
}
