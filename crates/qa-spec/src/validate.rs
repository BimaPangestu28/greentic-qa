use regex::Regex;
use serde_json::Value;

use crate::answers::{ValidationError, ValidationResult};
use crate::spec::form::FormSpec;
use crate::spec::question::{QuestionSpec, QuestionType};
use crate::visibility::{VisibilityMode, resolve_visibility};

pub fn validate(spec: &FormSpec, answers: &Value) -> ValidationResult {
    let visibility = resolve_visibility(spec, answers, VisibilityMode::Visible);
    let answers_map = answers.as_object().cloned().unwrap_or_default();

    let mut errors = Vec::new();
    let mut missing_required = Vec::new();

    for question in &spec.questions {
        if !visibility.get(&question.id).copied().unwrap_or(true) {
            continue;
        }

        match answers_map.get(&question.id) {
            None => {
                if question.required {
                    missing_required.push(question.id.clone());
                }
            }
            Some(value) => {
                if let Some(error) = validate_value(question, value) {
                    errors.push(error);
                }
            }
        }
    }

    let all_ids: std::collections::BTreeSet<_> = spec
        .questions
        .iter()
        .map(|question| question.id.clone())
        .collect();
    let unknown_fields: Vec<String> = answers_map
        .keys()
        .filter(|key| !all_ids.contains(*key))
        .cloned()
        .collect();

    ValidationResult {
        valid: errors.is_empty() && missing_required.is_empty() && unknown_fields.is_empty(),
        errors,
        missing_required,
        unknown_fields,
    }
}

fn validate_value(question: &QuestionSpec, value: &Value) -> Option<ValidationError> {
    if !matches_type(question, value) {
        return Some(ValidationError {
            question_id: Some(question.id.clone()),
            path: Some(format!("/{}", question.id)),
            message: "type mismatch".into(),
            code: Some("type_mismatch".into()),
        });
    }

    if let Some(constraint) = &question.constraint
        && let Some(error) = enforce_constraint(question, value, constraint)
    {
        return Some(error);
    }

    if matches!(question.kind, QuestionType::Enum)
        && let Some(choices) = &question.choices
        && let Some(text) = value.as_str()
        && !choices.contains(&text.to_string())
    {
        return Some(ValidationError {
            question_id: Some(question.id.clone()),
            path: Some(format!("/{}", question.id)),
            message: "invalid enum option".into(),
            code: Some("enum_mismatch".into()),
        });
    }

    None
}

fn matches_type(question: &QuestionSpec, value: &Value) -> bool {
    match question.kind {
        QuestionType::String | QuestionType::Enum => value.is_string(),
        QuestionType::Boolean => value.is_boolean(),
        QuestionType::Integer => value.is_i64(),
        QuestionType::Number => value.is_number(),
    }
}

fn enforce_constraint(
    question: &QuestionSpec,
    value: &Value,
    constraint: &crate::spec::question::Constraint,
) -> Option<ValidationError> {
    if let Some(pattern) = &constraint.pattern
        && let Some(text) = value.as_str()
        && let Ok(regex) = Regex::new(pattern)
        && !regex.is_match(text)
    {
        return Some(base_error(
            question,
            "value does not match pattern",
            "pattern_mismatch",
        ));
    }

    if let Some(min_len) = constraint.min_len
        && let Some(text) = value.as_str()
        && text.len() < min_len
    {
        return Some(base_error(
            question,
            "string shorter than min length",
            "min_length",
        ));
    }

    if let Some(max_len) = constraint.max_len
        && let Some(text) = value.as_str()
        && text.len() > max_len
    {
        return Some(base_error(
            question,
            "string longer than max length",
            "max_length",
        ));
    }

    if let Some(min) = constraint.min
        && let Some(value) = value.as_f64()
        && value < min
    {
        return Some(base_error(question, "value below minimum", "min"));
    }

    if let Some(max) = constraint.max
        && let Some(value) = value.as_f64()
        && value > max
    {
        return Some(base_error(question, "value above maximum", "max"));
    }

    None
}

fn base_error(question: &QuestionSpec, message: &str, code: &str) -> ValidationError {
    ValidationError {
        question_id: Some(question.id.clone()),
        path: Some(format!("/{}", question.id)),
        message: message.into(),
        code: Some(code.into()),
    }
}
