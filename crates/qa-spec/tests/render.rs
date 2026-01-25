use serde_json::json;

use qa_spec::{
    FormSpec,
    render::{RenderStatus, build_render_payload, render_card, render_json_ui, render_text},
};

fn fixture(name: &str) -> &'static str {
    match name {
        "simple_form" => include_str!("../tests/fixtures/simple_form.json"),
        "graph_flow" => include_str!("../tests/fixtures/graph_flow.json"),
        _ => panic!("unknown fixture {}", name),
    }
}

#[test]
fn render_text_includes_next_question() {
    let spec: FormSpec = serde_json::from_str(fixture("simple_form")).expect("deserialize");
    let ctx = json!({});
    let answers = json!({});
    let payload = build_render_payload(&spec, &ctx, &answers);

    assert_eq!(payload.status, RenderStatus::NeedInput);
    assert_eq!(payload.next_question_id.as_deref(), Some("q1"));

    let text = render_text(&payload);
    assert!(text.contains("Next question"));
    assert!(text.contains("Visible questions"));
}

#[test]
fn render_json_ui_exposes_structure() {
    let spec: FormSpec = serde_json::from_str(fixture("simple_form")).expect("deserialize");
    let ctx = json!({});
    let answers = json!({ "q1": "test-corp" });
    let payload = build_render_payload(&spec, &ctx, &answers);

    let ui = render_json_ui(&payload);
    assert_eq!(ui["form_id"], "example-form");
    assert_eq!(ui["progress"]["total"], 2);
    let questions = ui["questions"].as_array().expect("questions array");
    assert!(questions.iter().any(|q| q["id"] == "q1"));
    assert!(matches!(questions[0]["visible"].as_bool(), Some(true)));
}

#[test]
fn render_card_includes_patch_action() {
    let spec: FormSpec = serde_json::from_str(fixture("simple_form")).expect("deserialize");
    let ctx = json!({});
    let answers = json!({});
    let payload = build_render_payload(&spec, &ctx, &answers);

    let card = render_card(&payload);
    assert_eq!(card["version"], "1.3");
    let actions = card["actions"].as_array().expect("actions");
    assert_eq!(actions[0]["type"], "Action.Submit");
    assert_eq!(actions[0]["data"]["qa"]["mode"], "patch");
}

#[test]
fn render_card_uses_choice_input_for_enum() {
    let spec: FormSpec = serde_json::from_value(json!({
        "id": "enum-form",
        "title": "Enum Form",
        "version": "1.0",
        "questions": [
            {
                "id": "q_enum",
                "type": "enum",
                "title": "Choose option",
                "choices": ["alpha", "beta"],
                "required": true
            }
        ]
    }))
    .expect("deserialize");
    let ctx = json!({});
    let answers = json!({ "q1": "example-q1" });
    let payload = build_render_payload(&spec, &ctx, &answers);

    let card = render_card(&payload);
    let body = card["body"].as_array().expect("body");
    let container = body
        .iter()
        .find(|item| item["type"] == "Container")
        .expect("question container");
    let items = container["items"].as_array().expect("items");
    assert!(
        items
            .iter()
            .any(|item| item["type"].as_str() == Some("Input.ChoiceSet"))
    );
}
