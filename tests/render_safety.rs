use std::path::Path;
use veil_boundary_lab::{
    evaluate_suite, load_suite, render::render_html, report::ExpectationCheck,
};

#[test]
fn hostile_text_is_escaped_and_mismatch_disables_marks() {
    let mut report = evaluate_suite(load_suite(Path::new("fixtures/demo")).unwrap());
    let hostile = "</script><img src=x onerror=alert(1)>\"'&日本語";
    let case = &mut report.cases[0];
    case.input.title = hostile.into();
    case.input.lesson = hostile.into();
    case.input.response = hostile.into();
    case.input.context = hostile.into();
    case.input.policy = hostile.into();
    let html = render_html(&report).unwrap();
    assert!(!html.contains("<img"));
    assert!(!html.contains("</script>"));
    assert!(
        html.contains("&lt;/script&gt;&lt;img src=x onerror=alert(1)&gt;&quot;&#39;&amp;日本語")
    );
    assert!(!html.contains("<script"));
    for case in &mut report.cases {
        case.expectation_check = ExpectationCheck::Mismatched;
    }
    report.summary.mismatched = 5;
    let html = render_html(&report).unwrap();
    assert!(!html.contains("<mark "));
    assert!(html.contains("role=alert"));
}

#[test]
fn record_byte_and_parser_boundaries_are_from_core() {
    use veil_boundary_lab::engine::evaluate_case;
    let policies: Vec<leak_fence_core::Policy> =
        serde_json::from_str(include_str!("../fixtures/demo/policy.json")).unwrap();
    let mut policy = policies.into_iter().next().unwrap();
    let context = serde_json::from_str(include_str!("../fixtures/demo/context.json")).unwrap();
    let row = r#"{"id":"1","tenant_id":"synthetic-acme","name":"日本語"}"#;
    policy.max_bytes = row.len();
    assert_eq!(
        evaluate_case(row.as_bytes(), &policy, &context).code(),
        "passed"
    );
    assert_eq!(
        evaluate_case(format!("{row} ").as_bytes(), &policy, &context).code(),
        "byte_limit"
    );
    policy.max_bytes = 16384;
    policy.max_records = 1;
    assert_eq!(
        evaluate_case(format!("[{row}]").as_bytes(), &policy, &context).code(),
        "passed"
    );
    assert_eq!(
        evaluate_case(format!("[{row},{row}]").as_bytes(), &policy, &context).code(),
        "record_limit"
    );
    for (body, reason) in [
        ("[]", "passed"),
        ("{", "invalid_json"),
        (
            r#"{"id":"1","tenant_id":"synthetic-acme","name":{}}"#,
            "nested_value",
        ),
        (
            r#"{"id":"3","tenant_id":"other","extra":1}"#,
            "tenant_mismatch",
        ),
    ] {
        assert_eq!(
            evaluate_case(body.as_bytes(), &policy, &context).code(),
            reason
        );
    }
}
