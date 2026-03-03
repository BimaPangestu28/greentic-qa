//! Pure-functional helpers extracted from operator onboard logic.
//!
//! These utilities build [`FormSpec`]s, parse request parameters, and
//! construct payload JSON without any I/O or orchestration dependencies.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::spec::{Constraint, FormSpec, QuestionSpec, QuestionType};

// ── QaMode ──────────────────────────────────────────────────────────────────

/// The mode a QA session operates in.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum QaMode {
    Default,
    Setup,
    Upgrade,
    Remove,
}

impl QaMode {
    pub fn as_str(self) -> &'static str {
        match self {
            QaMode::Default => "default",
            QaMode::Setup => "setup",
            QaMode::Upgrade => "upgrade",
            QaMode::Remove => "remove",
        }
    }
}

/// Parse a [`QaMode`] from a JSON body's `"mode"` field.
pub fn parse_mode(body: &Value) -> QaMode {
    match body["mode"].as_str().unwrap_or("setup") {
        "upgrade" => QaMode::Upgrade,
        "remove" => QaMode::Remove,
        "default" => QaMode::Default,
        _ => QaMode::Setup,
    }
}

// ── FormSpec helpers ────────────────────────────────────────────────────────

/// Append a synthetic question to `form_spec` if no question with `key`
/// already exists.
pub fn push_synthetic_question(form_spec: &mut FormSpec, key: &str, secret: bool) {
    if form_spec.questions.iter().any(|q| q.id == key) {
        return;
    }
    form_spec.questions.push(QuestionSpec {
        id: key.to_string(),
        kind: QuestionType::String,
        title: key.to_string(),
        title_i18n: None,
        description: None,
        description_i18n: None,
        required: false,
        choices: None,
        default_value: None,
        secret,
        visible_if: None,
        constraint: None,
        list: None,
        computed: None,
        policy: Default::default(),
        computed_overridable: false,
    });
}

/// Build a minimal [`FormSpec`] by introspecting keys in a JSON config value.
pub fn make_minimal_form_spec(provider_id: &str, config: &Value) -> FormSpec {
    let questions = config
        .as_object()
        .map(|map| {
            map.keys()
                .map(|key| {
                    let (kind, secret, _) = infer_question_properties(key);
                    QuestionSpec {
                        id: key.clone(),
                        kind,
                        title: key.clone(),
                        title_i18n: None,
                        description: None,
                        description_i18n: None,
                        required: false,
                        choices: None,
                        default_value: None,
                        secret,
                        visible_if: None,
                        constraint: None,
                        list: None,
                        computed: None,
                        policy: Default::default(),
                        computed_overridable: false,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    FormSpec {
        id: format!("{provider_id}-setup"),
        title: format!("{provider_id} setup"),
        version: "1.0.0".to_string(),
        description: None,
        presentation: None,
        progress_policy: None,
        secrets_policy: None,
        store: vec![],
        validations: vec![],
        includes: vec![],
        questions,
    }
}

/// Infer [`QuestionType`], secret flag, and optional [`Constraint`] from a
/// question id using naming conventions.
pub fn infer_question_properties(id: &str) -> (QuestionType, bool, Option<Constraint>) {
    match id {
        "enabled" => (QuestionType::Boolean, false, None),
        id if id.ends_with("_url") || id == "public_base_url" || id == "api_base_url" => (
            QuestionType::String,
            false,
            Some(Constraint {
                pattern: Some(r"^https?://\S+".to_string()),
                min: None,
                max: None,
                min_len: None,
                max_len: None,
            }),
        ),
        id if id.ends_with("_token") || id.contains("secret") || id.contains("password") => {
            (QuestionType::String, true, None)
        }
        _ => (QuestionType::String, false, None),
    }
}

// ── Setup flow helpers ──────────────────────────────────────────────────────

/// Construct the JSON payload for a provider's `setup_default` flow invocation.
pub fn build_setup_flow_input(
    pack_id: &str,
    tenant: &str,
    team: Option<&str>,
    public_base_url: Option<&str>,
    config: &Value,
) -> Value {
    let team_str = team.unwrap_or("_");
    let mut payload = json!({
        "id": pack_id,
        "tenant": tenant,
        "team": team_str,
        "env": "dev",
    });
    let mut cfg = config.clone();
    if let Some(url) = public_base_url {
        payload["public_base_url"] = Value::String(url.to_string());
        if let Some(map) = cfg.as_object_mut() {
            map.entry("public_base_url".to_string())
                .or_insert_with(|| Value::String(url.to_string()));
        }
    }
    if let Some(map) = cfg.as_object_mut() {
        map.entry("id".to_string())
            .or_insert_with(|| Value::String(pack_id.to_string()));
    }
    payload["config"] = cfg;
    payload["msg"] = json!({
        "channel": "setup",
        "id": format!("{pack_id}.setup"),
        "message": {
            "id": format!("{pack_id}.setup_default__collect"),
            "text": "Collect inputs for setup_default."
        },
        "metadata": {},
        "reply_scope": "",
        "session_id": "setup",
        "tenant_id": tenant,
        "text": "Collect inputs for setup_default.",
        "user_id": "operator"
    });
    payload["payload"] = json!({
        "id": format!("{pack_id}-setup_default"),
        "spec_ref": "assets/setup.yaml"
    });
    payload["setup_answers"] = config.clone();
    if let Ok(answers_str) = serde_json::to_string(config) {
        payload["answers_json"] = Value::String(answers_str);
    }
    payload
}

// ── Path helpers ────────────────────────────────────────────────────────────

/// Resolve the gmap file path for a tenant/team.
pub fn resolve_gmap_path(
    bundle_root: &Path,
    tenant: &str,
    team: Option<&str>,
) -> PathBuf {
    match team {
        Some(team) if team != "_" => bundle_root
            .join("tenants")
            .join(tenant)
            .join("teams")
            .join(team)
            .join("team.gmap"),
        _ => bundle_root
            .join("tenants")
            .join(tenant)
            .join("tenant.gmap"),
    }
}

// ── String helpers ──────────────────────────────────────────────────────────

/// Validate an identifier: non-empty, lowercase alphanumeric + hyphens,
/// no leading/trailing hyphens.
pub fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
}

/// Capitalize the first character of a string.
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => format!("{}{}", c.to_ascii_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mode_defaults_to_setup() {
        assert_eq!(parse_mode(&json!({})), QaMode::Setup);
        assert_eq!(parse_mode(&json!({"mode": "setup"})), QaMode::Setup);
    }

    #[test]
    fn parse_mode_all_variants() {
        assert_eq!(parse_mode(&json!({"mode": "upgrade"})), QaMode::Upgrade);
        assert_eq!(parse_mode(&json!({"mode": "remove"})), QaMode::Remove);
        assert_eq!(parse_mode(&json!({"mode": "default"})), QaMode::Default);
    }

    #[test]
    fn valid_identifiers() {
        assert!(is_valid_identifier("hello"));
        assert!(is_valid_identifier("my-tenant"));
        assert!(is_valid_identifier("a1"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("-bad"));
        assert!(!is_valid_identifier("bad-"));
        assert!(!is_valid_identifier("has space"));
    }

    #[test]
    fn capitalize_works() {
        assert_eq!(capitalize("telegram"), "Telegram");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("A"), "A");
    }

    #[test]
    fn infer_properties_token_is_secret() {
        let (_, secret, _) = infer_question_properties("bot_token");
        assert!(secret);
    }

    #[test]
    fn infer_properties_url_has_constraint() {
        let (_, _, constraint) = infer_question_properties("webhook_url");
        assert!(constraint.is_some());
    }

    #[test]
    fn infer_properties_enabled_is_boolean() {
        let (kind, _, _) = infer_question_properties("enabled");
        assert_eq!(kind, QuestionType::Boolean);
    }

    #[test]
    fn minimal_form_spec_from_config() {
        let config = json!({"bot_token": "abc", "enabled": true});
        let spec = make_minimal_form_spec("messaging-telegram", &config);
        assert_eq!(spec.id, "messaging-telegram-setup");
        assert_eq!(spec.questions.len(), 2);
    }

    #[test]
    fn push_synthetic_question_deduplicates() {
        let mut spec = FormSpec {
            id: "test".into(),
            title: "test".into(),
            version: "1.0.0".into(),
            description: None,
            presentation: None,
            progress_policy: None,
            secrets_policy: None,
            store: vec![],
            validations: vec![],
            includes: vec![],
            questions: vec![],
        };
        push_synthetic_question(&mut spec, "key1", true);
        push_synthetic_question(&mut spec, "key1", true);
        assert_eq!(spec.questions.len(), 1);
    }

    #[test]
    fn resolve_gmap_path_tenant() {
        let p = resolve_gmap_path(Path::new("/bundle"), "default", None);
        assert_eq!(p, PathBuf::from("/bundle/tenants/default/tenant.gmap"));
    }

    #[test]
    fn resolve_gmap_path_team() {
        let p = resolve_gmap_path(Path::new("/bundle"), "default", Some("sales"));
        assert_eq!(
            p,
            PathBuf::from("/bundle/tenants/default/teams/sales/team.gmap")
        );
    }

    #[test]
    fn resolve_gmap_path_wildcard_team() {
        let p = resolve_gmap_path(Path::new("/bundle"), "default", Some("_"));
        assert_eq!(p, PathBuf::from("/bundle/tenants/default/tenant.gmap"));
    }

    #[test]
    fn build_setup_flow_input_basic() {
        let config = json!({"token": "abc"});
        let input =
            build_setup_flow_input("messaging-telegram", "default", None, None, &config);
        assert_eq!(input["id"], "messaging-telegram");
        assert_eq!(input["tenant"], "default");
        assert_eq!(input["team"], "_");
        assert_eq!(input["config"]["token"], "abc");
    }

    #[test]
    fn build_setup_flow_input_with_url() {
        let config = json!({"token": "abc"});
        let input = build_setup_flow_input(
            "messaging-telegram",
            "default",
            Some("sales"),
            Some("https://example.com"),
            &config,
        );
        assert_eq!(input["team"], "sales");
        assert_eq!(input["public_base_url"], "https://example.com");
        assert_eq!(
            input["config"]["public_base_url"],
            "https://example.com"
        );
    }
}
