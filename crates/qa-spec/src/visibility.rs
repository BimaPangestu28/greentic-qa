use serde_json::{Map, Value};

use crate::spec::form::FormSpec;

pub type VisibilityMap = std::collections::BTreeMap<String, bool>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityMode {
    Visible,
    Hidden,
    Error,
}

pub fn resolve_visibility(spec: &FormSpec, answers: &Value, mode: VisibilityMode) -> VisibilityMap {
    let mut map = VisibilityMap::new();
    let mut ctx_map = Map::new();
    ctx_map.insert("answers".into(), answers.clone());
    let ctx = Value::Object(ctx_map);

    for question in &spec.questions {
        let visible = if let Some(expr) = &question.visible_if {
            match expr.evaluate(&ctx) {
                Some(val) => val,
                None => match mode {
                    VisibilityMode::Visible => true,
                    VisibilityMode::Hidden => false,
                    VisibilityMode::Error => true,
                },
            }
        } else {
            true
        };
        map.insert(question.id.clone(), visible);
    }

    map
}
