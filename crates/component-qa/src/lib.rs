use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use thiserror::Error;

use qa_spec::{
    FormSpec, ProgressContext, RenderPayload, StoreContext, StoreError, VisibilityMode,
    answers_schema, build_render_payload, example_answers, next_question,
    render_card as qa_render_card, render_json_ui as qa_render_json_ui,
    render_text as qa_render_text, resolve_visibility, validate,
};

const DEFAULT_SPEC: &str = include_str!("../../qa-spec/tests/fixtures/simple_form.json");

#[derive(Debug, Error)]
enum ComponentError {
    #[error("failed to parse config/{0}")]
    ConfigParse(#[source] serde_json::Error),
    #[error("form '{0}' is not available")]
    FormUnavailable(String),
    #[error("json encode error: {0}")]
    JsonEncode(#[source] serde_json::Error),
    #[error("store apply failed: {0}")]
    Store(#[from] StoreError),
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct ComponentConfig {
    #[serde(default)]
    form_spec_json: Option<String>,
}

fn load_form_spec(config_json: &str) -> Result<FormSpec, ComponentError> {
    let config = if config_json.trim().is_empty() {
        ComponentConfig::default()
    } else {
        serde_json::from_str(config_json).map_err(ComponentError::ConfigParse)?
    };

    let spec_json = config.form_spec_json.as_deref().unwrap_or(DEFAULT_SPEC);

    serde_json::from_str(spec_json).map_err(ComponentError::ConfigParse)
}

fn parse_context(ctx_json: &str) -> Value {
    serde_json::from_str(ctx_json).unwrap_or_else(|_| Value::Object(Map::new()))
}

fn resolve_context_answers(ctx: &Value) -> Value {
    ctx.get("answers")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()))
}

fn parse_answers(answers_json: &str) -> Value {
    serde_json::from_str(answers_json).unwrap_or_else(|_| Value::Object(Map::new()))
}

fn secrets_host_available(ctx: &Value) -> bool {
    ctx.get("secrets_host_available")
        .and_then(Value::as_bool)
        .or_else(|| {
            ctx.get("config")
                .and_then(Value::as_object)
                .and_then(|config| config.get("secrets_host_available"))
                .and_then(Value::as_bool)
        })
        .unwrap_or(false)
}

fn respond(result: Result<Value, ComponentError>) -> String {
    match result {
        Ok(value) => serde_json::to_string(&value).unwrap_or_else(|error| {
            json!({"error": format!("json encode: {}", error)}).to_string()
        }),
        Err(err) => json!({ "error": err.to_string() }).to_string(),
    }
}

pub fn describe(form_id: &str, config_json: &str) -> String {
    respond(load_form_spec(config_json).and_then(|spec| {
        if spec.id != form_id {
            Err(ComponentError::FormUnavailable(form_id.to_string()))
        } else {
            serde_json::to_value(spec).map_err(ComponentError::JsonEncode)
        }
    }))
}

fn ensure_form(form_id: &str, config_json: &str) -> Result<FormSpec, ComponentError> {
    let spec = load_form_spec(config_json)?;
    if spec.id != form_id {
        Err(ComponentError::FormUnavailable(form_id.to_string()))
    } else {
        Ok(spec)
    }
}

pub fn get_answer_schema(form_id: &str, config_json: &str, ctx_json: &str) -> String {
    let schema = ensure_form(form_id, config_json).map(|spec| {
        let ctx = parse_context(ctx_json);
        let answers = resolve_context_answers(&ctx);
        let visibility = resolve_visibility(&spec, &answers, VisibilityMode::Visible);
        answers_schema(&spec, &visibility)
    });
    respond(schema)
}

pub fn get_example_answers(form_id: &str, config_json: &str, ctx_json: &str) -> String {
    let result = ensure_form(form_id, config_json).map(|spec| {
        let ctx = parse_context(ctx_json);
        let answers = resolve_context_answers(&ctx);
        let visibility = resolve_visibility(&spec, &answers, VisibilityMode::Visible);
        example_answers(&spec, &visibility)
    });
    respond(result)
}

pub fn validate_answers(form_id: &str, config_json: &str, answers_json: &str) -> String {
    let validation = ensure_form(form_id, config_json).and_then(|spec| {
        let answers = serde_json::from_str(answers_json).map_err(ComponentError::ConfigParse)?;
        serde_json::to_value(validate(&spec, &answers)).map_err(ComponentError::JsonEncode)
    });
    respond(validation)
}

pub fn next(form_id: &str, ctx_json: &str, answers_json: &str) -> String {
    let result = ensure_form(form_id, ctx_json).map(|spec| {
        let ctx = parse_context(ctx_json);
        let answers = parse_answers(answers_json);
        let visibility = resolve_visibility(&spec, &answers, VisibilityMode::Visible);
        let progress_ctx = ProgressContext::new(answers.clone(), &ctx);
        let next_q = next_question(&spec, &progress_ctx, &visibility);
        let answered = progress_ctx.answered_count(&spec, &visibility);
        let total = visibility.values().filter(|visible| **visible).count();
        json!({
            "status": if next_q.is_some() { "need_input" } else { "complete" },
            "next_question_id": next_q,
            "progress": {
                "answered": answered,
                "total": total
            }
        })
    });
    respond(result)
}

pub fn apply_store(form_id: &str, ctx_json: &str, answers_json: &str) -> String {
    let result = ensure_form(form_id, ctx_json).and_then(|spec| {
        let ctx = parse_context(ctx_json);
        let answers = parse_answers(answers_json);
        let mut store_ctx = StoreContext::from_value(&ctx);
        store_ctx.answers = answers;
        let host_available = secrets_host_available(&ctx);
        store_ctx.apply_ops(&spec.store, spec.secrets_policy.as_ref(), host_available)?;
        Ok(store_ctx.to_value())
    });
    respond(result)
}

fn render_payload(
    form_id: &str,
    config_json: &str,
    ctx_json: &str,
    answers_json: &str,
) -> Result<RenderPayload, ComponentError> {
    let spec = ensure_form(form_id, config_json)?;
    let ctx = parse_context(ctx_json);
    let answers = parse_answers(answers_json);
    Ok(build_render_payload(&spec, &ctx, &answers))
}

fn respond_string(result: Result<String, ComponentError>) -> String {
    match result {
        Ok(value) => value,
        Err(err) => json!({ "error": err.to_string() }).to_string(),
    }
}

pub fn render_text(form_id: &str, config_json: &str, ctx_json: &str, answers_json: &str) -> String {
    respond_string(
        render_payload(form_id, config_json, ctx_json, answers_json)
            .map(|payload| qa_render_text(&payload)),
    )
}

pub fn render_json_ui(
    form_id: &str,
    config_json: &str,
    ctx_json: &str,
    answers_json: &str,
) -> String {
    respond(
        render_payload(form_id, config_json, ctx_json, answers_json)
            .map(|payload| qa_render_json_ui(&payload)),
    )
}

pub fn render_card(form_id: &str, config_json: &str, ctx_json: &str, answers_json: &str) -> String {
    respond(
        render_payload(form_id, config_json, ctx_json, answers_json)
            .map(|payload| qa_render_card(&payload)),
    )
}

fn submission_progress(payload: &RenderPayload) -> Value {
    json!({
        "answered": payload.progress.answered,
        "total": payload.progress.total,
    })
}

fn build_error_response(
    payload: &RenderPayload,
    answers: Value,
    validation: &qa_spec::ValidationResult,
) -> Result<Value, ComponentError> {
    let validation_value = serde_json::to_value(validation).map_err(ComponentError::JsonEncode)?;
    Ok(json!({
        "status": "error",
        "next_question_id": payload.next_question_id,
        "progress": submission_progress(payload),
        "answers": answers,
        "validation": validation_value,
    }))
}

fn build_success_response(
    payload: &RenderPayload,
    answers: Value,
    store_ctx: &StoreContext,
) -> Value {
    let status = if payload.next_question_id.is_some() {
        "need_input"
    } else {
        "complete"
    };

    json!({
        "status": status,
        "next_question_id": payload.next_question_id,
        "progress": submission_progress(payload),
        "answers": answers,
        "store": store_ctx.to_value(),
    })
}

fn with_answers_mutated(answers_json: &str, question_id: &str, value: Value) -> Value {
    let mut map = parse_answers(answers_json)
        .as_object()
        .cloned()
        .unwrap_or_default();
    map.insert(question_id.to_string(), value);
    Value::Object(map)
}

pub fn submit_patch(
    form_id: &str,
    config_json: &str,
    ctx_json: &str,
    answers_json: &str,
    question_id: &str,
    value_json: &str,
) -> String {
    respond(ensure_form(form_id, config_json).and_then(|spec| {
        let ctx = parse_context(ctx_json);
        let value: Value = serde_json::from_str(value_json).map_err(ComponentError::ConfigParse)?;
        let answers = with_answers_mutated(answers_json, question_id, value);
        let validation = validate(&spec, &answers);
        let payload = build_render_payload(&spec, &ctx, &answers);

        if !validation.valid {
            return build_error_response(&payload, answers, &validation);
        }

        let mut store_ctx = StoreContext::from_value(&ctx);
        store_ctx.answers = answers.clone();
        let host_available = secrets_host_available(&ctx);
        store_ctx.apply_ops(&spec.store, spec.secrets_policy.as_ref(), host_available)?;
        let response = build_success_response(&payload, answers, &store_ctx);
        Ok(response)
    }))
}

pub fn submit_all(form_id: &str, config_json: &str, ctx_json: &str, answers_json: &str) -> String {
    respond(ensure_form(form_id, config_json).and_then(|spec| {
        let ctx = parse_context(ctx_json);
        let answers = parse_answers(answers_json);
        let validation = validate(&spec, &answers);
        let payload = build_render_payload(&spec, &ctx, &answers);

        if !validation.valid {
            return build_error_response(&payload, answers, &validation);
        }

        let mut store_ctx = StoreContext::from_value(&ctx);
        store_ctx.answers = answers.clone();
        let host_available = secrets_host_available(&ctx);
        store_ctx.apply_ops(&spec.store, spec.secrets_policy.as_ref(), host_available)?;
        let response = build_success_response(&payload, answers, &store_ctx);
        Ok(response)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn describe_returns_spec_json() {
        let payload = describe("example-form", "");
        let spec: Value = serde_json::from_str(&payload).expect("valid json");
        assert_eq!(spec["id"], "example-form");
    }

    #[test]
    fn schema_matches_questions() {
        let schema = get_answer_schema("example-form", "", "{}");
        let value: Value = serde_json::from_str(&schema).expect("json");
        assert!(
            value
                .get("properties")
                .unwrap()
                .as_object()
                .unwrap()
                .contains_key("q1")
        );
    }

    #[test]
    fn example_answers_include_question_values() {
        let examples = get_example_answers("example-form", "", "{}");
        let parsed: Value = serde_json::from_str(&examples).expect("json");
        assert_eq!(parsed["q1"], "example-q1");
    }

    #[test]
    fn validate_answers_reports_valid_when_complete() {
        let answers = json!({ "q1": "tester", "q2": true });
        let result = validate_answers("example-form", "", &answers.to_string());
        let parsed: Value = serde_json::from_str(&result).expect("json");
        assert!(parsed["valid"].as_bool().unwrap_or(false));
    }

    #[test]
    fn next_returns_progress_payload() {
        let spec = json!({
            "id": "progress-form",
            "title": "Progress",
            "version": "1.0",
            "progress_policy": {
                "skip_answered": true
            },
            "questions": [
                { "id": "q1", "type": "string", "title": "q1", "required": true },
                { "id": "q2", "type": "string", "title": "q2", "required": true }
            ]
        });
        let ctx = json!({ "form_spec_json": spec.to_string() });
        let response = next("progress-form", &ctx.to_string(), r#"{"q1": "test"}"#);
        let parsed: Value = serde_json::from_str(&response).expect("json");
        assert_eq!(parsed["status"], "need_input");
        assert_eq!(parsed["next_question_id"], "q2");
        assert_eq!(parsed["progress"]["answered"], 1);
    }

    #[test]
    fn apply_store_writes_state_value() {
        let spec = json!({
            "id": "store-form",
            "title": "Store",
            "version": "1.0",
            "questions": [
                { "id": "q1", "type": "string", "title": "q1", "required": true }
            ],
            "store": [
                {
                    "target": "state",
                    "path": "/flag",
                    "value": true
                }
            ]
        });
        let ctx = json!({
            "form_spec_json": spec.to_string(),
            "state": {}
        });
        let result = apply_store("store-form", &ctx.to_string(), "{}");
        let parsed: Value = serde_json::from_str(&result).expect("json");
        assert_eq!(parsed["state"]["flag"], true);
    }

    #[test]
    fn apply_store_writes_secret_when_allowed() {
        let spec = json!({
            "id": "store-secret",
            "title": "Store Secret",
            "version": "1.0",
            "questions": [
                { "id": "q1", "type": "string", "title": "q1", "required": true }
            ],
            "store": [
                {
                    "target": "secrets",
                    "path": "/aws/key",
                    "value": "value"
                }
            ],
            "secrets_policy": {
                "enabled": true,
                "read_enabled": true,
                "write_enabled": true,
                "allow": ["aws/*"]
            }
        });
        let ctx = json!({
            "form_spec_json": spec.to_string(),
            "state": {},
            "secrets_host_available": true
        });
        let result = apply_store("store-secret", &ctx.to_string(), "{}");
        let parsed: Value = serde_json::from_str(&result).expect("json");
        assert_eq!(parsed["secrets"]["aws"]["key"], "value");
    }

    #[test]
    fn render_text_outputs_summary() {
        let output = render_text("example-form", "", "{}", "{}");
        assert!(output.contains("Form:"));
        assert!(output.contains("Visible questions"));
    }

    #[test]
    fn render_json_ui_outputs_json_payload() {
        let payload = render_json_ui("example-form", "", "{}", r#"{"q1":"value"}"#);
        let parsed: Value = serde_json::from_str(&payload).expect("json");
        assert_eq!(parsed["form_id"], "example-form");
        assert_eq!(parsed["progress"]["total"], 2);
    }

    #[test]
    fn render_card_outputs_patch_action() {
        let payload = render_card("example-form", "", "{}", "{}");
        let parsed: Value = serde_json::from_str(&payload).expect("json");
        assert_eq!(parsed["version"], "1.3");
        let actions = parsed["actions"].as_array().expect("actions");
        assert_eq!(actions[0]["data"]["qa"]["mode"], "patch");
    }

    #[test]
    fn submit_patch_advances_and_updates_store() {
        let response = submit_patch("example-form", "", "{}", "{}", "q1", r#""Acme""#);
        let parsed: Value = serde_json::from_str(&response).expect("json");
        assert_eq!(parsed["status"], "need_input");
        assert_eq!(parsed["next_question_id"], "q2");
        assert_eq!(parsed["answers"]["q1"], "Acme");
        assert_eq!(parsed["store"]["answers"]["q1"], "Acme");
    }

    #[test]
    fn submit_patch_returns_validation_error() {
        let response = submit_patch("example-form", "", "{}", "{}", "q1", "true");
        let parsed: Value = serde_json::from_str(&response).expect("json");
        assert_eq!(parsed["status"], "error");
        assert_eq!(parsed["validation"]["errors"][0]["code"], "type_mismatch");
    }

    #[test]
    fn submit_all_completes_with_valid_answers() {
        let response = submit_all("example-form", "", "{}", r#"{"q1":"Acme","q2":true}"#);
        let parsed: Value = serde_json::from_str(&response).expect("json");
        assert_eq!(parsed["status"], "complete");
        assert!(parsed["next_question_id"].is_null());
        assert_eq!(parsed["answers"]["q2"], true);
        assert_eq!(parsed["store"]["answers"]["q2"], true);
    }
}
