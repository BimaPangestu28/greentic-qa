use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Lightweight expression AST used for `visible_if` and decisions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Expr {
    LiteralBool { value: bool },
    Eq { left: String, right: String },
    And { expressions: Vec<Expr> },
    Or { expressions: Vec<Expr> },
    Not { expression: Box<Expr> },
    Var { path: String },
}

impl Expr {
    fn get_value<'a>(ctx: &'a Value, path: &str) -> Option<&'a Value> {
        ctx.pointer(path)
    }

    /// Evaluates the expression to a boolean if possible.
    pub fn evaluate(&self, ctx: &Value) -> Option<bool> {
        match self {
            Expr::LiteralBool { value } => Some(*value),
            Expr::Eq { left, right } => {
                let left_val = Self::get_value(ctx, left)?;
                let right_val = Self::get_value(ctx, right)?;
                Some(left_val == right_val)
            }
            Expr::And { expressions } => {
                for expr in expressions {
                    match expr.evaluate(ctx) {
                        Some(true) => continue,
                        Some(false) => return Some(false),
                        None => return None,
                    }
                }
                Some(true)
            }
            Expr::Or { expressions } => {
                for expr in expressions {
                    if let Some(true) = expr.evaluate(ctx) {
                        return Some(true);
                    }
                }
                Some(false)
            }
            Expr::Not { expression } => expression.evaluate(ctx).map(|value| !value),
            Expr::Var { path } => Self::get_value(ctx, path).and_then(|v| v.as_bool()),
        }
    }
}
