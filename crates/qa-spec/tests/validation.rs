use serde_json::{Value, json};

use qa_spec::{
    VisibilityMap, VisibilityMode, answers_schema, example_answers, resolve_visibility, validate,
};

use qa_spec::spec::form::FormSpec;
use qa_spec::spec::question::{QuestionSpec, QuestionType};

fn make_simple_form() -> FormSpec {
    FormSpec {
        id: "simple".into(),
        title: "Simple".into(),
        version: "1.0.0".into(),
        description: None,
        presentation: None,
        progress_policy: None,
        secrets_policy: None,
        store: vec![],
        questions: vec![
            QuestionSpec {
                id: "name".into(),
                kind: QuestionType::String,
                title: "Name".into(),
                description: None,
                required: true,
                choices: None,
                default_value: None,
                secret: false,
                visible_if: None,
                constraint: None,
                policy: Default::default(),
            },
            QuestionSpec {
                id: "flag".into(),
                kind: QuestionType::Boolean,
                title: "flag".into(),
                description: None,
                required: false,
                choices: None,
                default_value: None,
                secret: false,
                visible_if: None,
                constraint: None,
                policy: Default::default(),
            },
        ],
    }
}

#[test]
fn schema_contains_required_properties() {
    let spec = make_simple_form();
    let visibility = resolve_visibility(&spec, &json!({}), VisibilityMode::Visible);
    let schema = answers_schema(&spec, &visibility);
    let props = schema.get("properties").unwrap().as_object().unwrap();
    assert!(props.contains_key("name"));
    assert!(props.contains_key("flag"));
    let required = schema.get("required").unwrap().as_array().unwrap();
    assert!(required.iter().any(|value| value.as_str() == Some("name")));
}

#[test]
fn example_answers_include_questions() {
    let spec = make_simple_form();
    let visibility = VisibilityMap::from([("name".into(), true), ("flag".into(), true)]);
    let examples = example_answers(&spec, &visibility);
    assert_eq!(examples["name"], Value::String("example-name".into()));
    assert_eq!(examples["flag"], Value::Bool(false));
}

#[test]
fn validation_reports_missing() {
    let spec = make_simple_form();
    let answers: Value = json!({});
    let result = validate(&spec, &answers);
    assert!(!result.valid);
    assert_eq!(result.missing_required, vec!["name"]);
}
