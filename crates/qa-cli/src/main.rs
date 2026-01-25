pub mod builder;

use builder::{
    CliQuestionType, FormInput, GenerationInput, QuestionInput, build_bundle, write_bundle,
};
use clap::{Parser, Subcommand};
use component_qa::{next as qa_next, render_json_ui, render_text, submit_patch};
use qa_spec::{FormSpec, ValidationResult, validate};
use serde_json::{Map, Number, Value, json};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

type CliResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Text-based QA wizard CLI",
    long_about = "Provides wizard helpers, spec generation, and validation helpers backed by the QA component"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the existing QA wizard flow in a text shell.
    Wizard {
        /// Path to the FormSpec JSON describing the wizard.
        #[arg(long, value_name = "SPEC")]
        spec: PathBuf,
        /// Optional JSON file containing initial answers.
        #[arg(long, value_name = "ANSWERS")]
        answers: Option<PathBuf>,
    },
    /// Interactive form generator that creates a bundle of derived artifacts.
    New {
        /// Root directory where the generated bundle will be emitted (defaults to QA_WIZARD_OUTPUT_DIR or current working directory).
        #[arg(long, value_name = "DIR")]
        out: Option<PathBuf>,
        /// Overwrite existing bundle if present.
        #[arg(long)]
        force: bool,
    },
    /// Non-interactive generator that consumes JSON answers and emits the bundle.
    Generate {
        /// JSON file describing the form metadata + questions.
        #[arg(long, value_name = "INPUT")]
        input: PathBuf,
        /// Root directory where the generated bundle will be emitted.
        #[arg(long, value_name = "DIR")]
        out: Option<PathBuf>,
        /// Overwrite existing bundle if present.
        #[arg(long)]
        force: bool,
    },
    /// Validate answers against a generated FormSpec.
    Validate {
        /// Path to the FormSpec JSON.
        #[arg(long, value_name = "SPEC")]
        spec: PathBuf,
        /// Path to the answers JSON file.
        #[arg(long, value_name = "ANSWERS")]
        answers: PathBuf,
    },
}

fn main() -> CliResult<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Wizard { spec, answers } => run_wizard(spec, answers),
        Command::New { out, force } => run_new(out, force),
        Command::Generate { input, out, force } => run_generate(input, out, force),
        Command::Validate { spec, answers } => run_validate(spec, answers),
    }
}

fn run_new(out_dir: Option<PathBuf>, force: bool) -> CliResult<()> {
    println!("Interactive QA form generator");
    let form_id = prompt_non_empty("Form ID (dot-delimited)", None)?;
    let title = prompt_non_empty("Form title", None)?;
    let version = prompt_non_empty("Form version", Some("0.1.0"))?;
    let description = prompt_optional("Description (optional)")?;
    let summary = prompt_optional("Summary for README (optional)")?;
    let dir_name = prompt_non_empty("Output directory name", Some(&form_id))?;

    let mut questions = Vec::new();
    loop {
        let question_id = prompt_optional("Question ID (blank to finish)")?;
        let question_id = match question_id.filter(|value| !value.trim().is_empty()) {
            Some(id) => {
                if questions
                    .iter()
                    .any(|question: &QuestionInput| question.id == id)
                {
                    println!(
                        "Question ID '{}' already used; choose a different identifier.",
                        id
                    );
                    continue;
                }
                id
            }
            None => break,
        };

        let question_title = prompt_non_empty("Question title", Some(&question_id))?;
        let kind = prompt_question_type()?;
        let required = prompt_bool("Required? (Y/n)", true)?;
        let question_description = prompt_optional("Question description (optional)")?;
        let default_value = prompt_optional("Default value (optional)")?;
        let choices = if matches!(kind, CliQuestionType::Enum) {
            Some(prompt_enum_choices()?)
        } else {
            None
        };
        let secret = prompt_bool("Secret value? (y/N)", false)?;

        questions.push(QuestionInput {
            id: question_id,
            kind,
            title: question_title,
            description: question_description,
            required,
            default_value,
            choices,
            secret,
        });
    }

    if questions.is_empty() {
        return Err("at least one question is required".into());
    }

    let input = GenerationInput {
        dir_name,
        summary_md: summary,
        form: FormInput {
            id: form_id,
            title,
            version,
            description,
            progress_policy: None,
        },
        questions,
    };

    let out_root = resolve_output_root(out_dir)?;
    let bundle_dir = out_root.join(&input.dir_name);
    ensure_allowed_root(&bundle_dir)?;
    if bundle_dir.exists() {
        if force {
            fs::remove_dir_all(&bundle_dir)?;
        } else {
            return Err(format!(
                "bundle {} already exists; rerun with --force to overwrite",
                bundle_dir.display()
            )
            .into());
        }
    }

    let bundle = build_bundle(&input)?;
    let bundle_dir = write_bundle(&bundle, &input, &out_root)?;
    println!("Generated QA bundle at {}", bundle_dir.display());
    Ok(())
}

fn run_generate(input_path: PathBuf, out_dir: Option<PathBuf>, force: bool) -> CliResult<()> {
    let contents = fs::read_to_string(&input_path)?;
    let input: GenerationInput = serde_json::from_str(&contents)?;
    let out_root = resolve_output_root(out_dir)?;
    let bundle_dir = out_root.join(&input.dir_name);
    ensure_allowed_root(&bundle_dir)?;
    if bundle_dir.exists() {
        if force {
            fs::remove_dir_all(&bundle_dir)?;
        } else {
            return Err(format!(
                "bundle {} already exists; rerun with --force to overwrite",
                bundle_dir.display()
            )
            .into());
        }
    }

    let bundle = build_bundle(&input)?;
    let bundle_dir = write_bundle(&bundle, &input, &out_root)?;
    println!("Generated QA bundle at {}", bundle_dir.display());
    Ok(())
}

fn run_validate(spec_path: PathBuf, answers_path: PathBuf) -> CliResult<()> {
    let spec_json = fs::read_to_string(spec_path)?;
    let spec: FormSpec = serde_json::from_str(&spec_json)?;
    let answers_json = fs::read_to_string(answers_path)?;
    let answers: Value = serde_json::from_str(&answers_json)?;

    let result = validate(&spec, &answers);
    println!(
        "Validation result: {}",
        if result.valid { "valid" } else { "invalid" }
    );
    describe_validation(&result);

    if result.valid {
        Ok(())
    } else {
        Err("validation failed".into())
    }
}

fn describe_validation(result: &ValidationResult) {
    if !result.errors.is_empty() {
        println!("Errors:");
        for error in &result.errors {
            println!(
                "  {} - {}",
                error.path.as_deref().unwrap_or("<unknown>"),
                error.message
            );
        }
    }
    if !result.missing_required.is_empty() {
        println!(
            "Missing required answers: {}",
            result.missing_required.join(", ")
        );
    }
    if !result.unknown_fields.is_empty() {
        println!(
            "Unknown answer fields: {}",
            result.unknown_fields.join(", ")
        );
    }
}

fn resolve_output_root(out: Option<PathBuf>) -> CliResult<PathBuf> {
    let candidate = match out {
        Some(path) => path,
        None => env::var_os("QA_WIZARD_OUTPUT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".")),
    };
    if candidate.as_os_str().is_empty() {
        return Err("output directory cannot be empty".into());
    }
    ensure_allowed_root(&candidate)?;
    Ok(candidate)
}

fn ensure_allowed_root(target: &Path) -> CliResult<()> {
    let target = canonicalize_target(target)?;
    let roots = allowed_roots()?;
    if roots.iter().any(|root| target.starts_with(root)) {
        Ok(())
    } else {
        Err(format!(
            "path '{}' is outside allowed roots {:?}",
            target.display(),
            roots
        )
        .into())
    }
}

fn allowed_roots() -> CliResult<Vec<PathBuf>> {
    let roots = env::var("QA_WIZARD_ALLOWED_ROOTS")
        .ok()
        .map(|value| {
            value
                .split(':')
                .filter_map(|segment| {
                    let trimmed = segment.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(trimmed))
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut canonical_roots = Vec::new();
    for root in roots {
        if let Ok(canonical) = root.canonicalize() {
            canonical_roots.push(canonical);
        } else {
            canonical_roots.push(root);
        }
    }

    if canonical_roots.is_empty() {
        canonical_roots.push(env::current_dir()?);
    }

    Ok(canonical_roots)
}

fn canonicalize_target(path: &Path) -> CliResult<PathBuf> {
    if path.exists() {
        return Ok(path.canonicalize()?);
    }

    if let Some(parent) = path.parent()
        && let Ok(parent_canon) = parent.canonicalize()
    {
        if let Some(file_name) = path.file_name() {
            return Ok(parent_canon.join(file_name));
        } else {
            return Ok(parent_canon);
        }
    }

    let cwd = env::current_dir()?;
    Ok(cwd.join(path))
}

fn run_wizard(spec_path: PathBuf, answers_path: Option<PathBuf>) -> CliResult<()> {
    let spec_str = fs::read_to_string(&spec_path)?;
    let spec_value: Value = serde_json::from_str(&spec_str)?;
    let form_id = spec_value
        .get("id")
        .and_then(Value::as_str)
        .ok_or("form spec is missing an id")?;
    let config_json = json!({ "form_spec_json": spec_str }).to_string();

    let mut answers = if let Some(path) = answers_path {
        let contents = fs::read_to_string(path)?;
        serde_json::from_str(&contents)?
    } else {
        Value::Object(Map::new())
    };

    loop {
        let answers_str = answers.to_string();
        println!("{}", render_text(form_id, &config_json, "{}", &answers_str));

        let next_value = parse_component_result(&qa_next(form_id, &config_json, &answers_str))?;
        if next_value["status"] == "complete" {
            break;
        }
        let question_id = next_value["next_question_id"]
            .as_str()
            .ok_or("wizard failed to return a next question")?;

        let ui =
            parse_component_result(&render_json_ui(form_id, &config_json, "{}", &answers_str))?;
        let question = find_question(&ui, question_id)?;
        let answer = prompt_question(question)?;

        let value_json = serde_json::to_string(&answer)?;
        let submit_value = parse_component_result(&submit_patch(
            form_id,
            &config_json,
            "{}",
            &answers_str,
            question_id,
            &value_json,
        ))?;

        if submit_value["status"] == "error" {
            print_validation_errors(&submit_value)?;
            continue;
        }

        answers = submit_value["answers"].clone();
        if submit_value["status"] == "complete" {
            break;
        }
    }

    println!("Wizard complete! Final answers:\n{}", answers);
    Ok(())
}

fn parse_component_result(response: &str) -> CliResult<Value> {
    let value: Value = serde_json::from_str(response)?;
    if let Some(error) = value.get("error").and_then(Value::as_str) {
        Err(error.into())
    } else {
        Ok(value)
    }
}

fn find_question(ui: &Value, question_id: &str) -> CliResult<Value> {
    let question = ui
        .get("questions")
        .and_then(Value::as_array)
        .and_then(|questions| {
            questions
                .iter()
                .find(|question| question["id"].as_str() == Some(question_id))
                .cloned()
        })
        .ok_or_else(|| format!("question '{}' not found", question_id))?;
    Ok(question)
}

fn prompt_question(question: Value) -> CliResult<Value> {
    loop {
        if let Some(title) = question.get("title").and_then(Value::as_str) {
            println!("Question: {}", title);
        }
        if let Some(description) = question.get("description").and_then(Value::as_str) {
            println!("{}", description);
        }
        if let Some(choices) = question.get("choices").and_then(Value::as_array)
            && !choices.is_empty()
        {
            println!(
                "Choices: {}",
                choices
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().eq_ignore_ascii_case("exit") {
            return Err("wizard aborted by user".into());
        }

        match parse_answer(&question, input.trim()) {
            Ok(value) => return Ok(value),
            Err(err) => {
                eprintln!("Invalid answer: {}", err);
            }
        }
    }
}

fn parse_answer(question: &Value, raw: &str) -> Result<Value, String> {
    let prompt_value = if raw.is_empty() {
        question
            .get("default")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string()
    } else {
        raw.trim().to_string()
    };

    if prompt_value.is_empty() {
        if !question
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        {
            return Ok(Value::Null);
        }
        return Err("answer required".into());
    }

    match question
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("string")
    {
        "boolean" => parse_boolean(&prompt_value),
        "integer" => parse_integer(&prompt_value),
        "number" => parse_number(&prompt_value),
        "enum" => parse_enum(question, &prompt_value),
        _ => Ok(Value::String(prompt_value)),
    }
}

fn parse_boolean(raw: &str) -> Result<Value, String> {
    match raw.to_lowercase().as_str() {
        "true" | "t" | "yes" | "y" | "1" => Ok(Value::Bool(true)),
        "false" | "f" | "no" | "n" | "0" => Ok(Value::Bool(false)),
        _ => Err("expected boolean (y/n/true/false)".into()),
    }
}

fn parse_integer(raw: &str) -> Result<Value, String> {
    raw.parse::<i64>()
        .map(Number::from)
        .map(Value::Number)
        .map_err(|_| "expected integer".into())
}

fn parse_number(raw: &str) -> Result<Value, String> {
    raw.parse::<f64>()
        .map_err(|_| "expected number".into())
        .and_then(|value| {
            serde_json::Number::from_f64(value)
                .map(Value::Number)
                .ok_or_else(|| "number must be finite".into())
        })
}

fn parse_enum(question: &Value, raw: &str) -> Result<Value, String> {
    let choices = question
        .get("choices")
        .and_then(Value::as_array)
        .ok_or_else(|| "enum choices missing".to_string())?;

    let allowed = choices.iter().filter_map(Value::as_str).collect::<Vec<_>>();

    if allowed.iter().any(|choice| choice == &raw) {
        Ok(Value::String(raw.to_string()))
    } else {
        Err(format!("answer must be one of: {}", allowed.join(", ")))
    }
}

fn prompt_line(prompt: &str, default: Option<&str>) -> CliResult<String> {
    if let Some(default_value) = default {
        print!("{} [{}]: ", prompt, default_value);
    } else {
        print!("{}: ", prompt);
    }
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        if let Some(default_value) = default {
            Ok(default_value.to_string())
        } else {
            Ok(String::new())
        }
    } else {
        Ok(trimmed.to_string())
    }
}

fn prompt_optional(prompt: &str) -> CliResult<Option<String>> {
    let value = prompt_line(prompt, None)?;
    if value.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

fn prompt_non_empty(prompt: &str, default: Option<&str>) -> CliResult<String> {
    loop {
        let value = prompt_line(prompt, default)?;
        if !value.trim().is_empty() {
            return Ok(value);
        }
        println!("Value cannot be empty.");
    }
}

fn prompt_bool(prompt: &str, default: bool) -> CliResult<bool> {
    let default_hint = if default { "Y" } else { "N" };
    loop {
        let line = prompt_line(prompt, Some(default_hint))?;
        match line.trim().to_lowercase().as_str() {
            "" => return Ok(default),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            other => {
                println!("Invalid answer '{}'. Use 'y' or 'n'.", other);
            }
        }
    }
}

fn prompt_question_type() -> CliResult<CliQuestionType> {
    loop {
        let value = prompt_line(
            "Question type (string|boolean|integer|number|enum)",
            Some("string"),
        )?;
        match CliQuestionType::from_str(&value) {
            Ok(kind) => return Ok(kind),
            Err(err) => println!("{}", err),
        }
    }
}

fn prompt_enum_choices() -> CliResult<Vec<String>> {
    loop {
        let raw = prompt_line("Comma separated choices", None)?;
        let normalized = raw
            .split(',')
            .map(str::trim)
            .filter(|choice| !choice.is_empty())
            .map(|choice| choice.to_string())
            .collect::<Vec<_>>();
        if normalized.is_empty() {
            println!("Provide at least one choice for enum questions.");
            continue;
        }
        return Ok(normalized);
    }
}

fn print_validation_errors(response: &Value) -> CliResult<()> {
    if let Some(errors) = response
        .get("validation")
        .and_then(|value| value.get("errors"))
        .and_then(Value::as_array)
    {
        eprintln!("Validation errors:");
        for error in errors {
            let path = error
                .get("path")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>");
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("validation failed");
            eprintln!("  {}: {}", path, message);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    use crate::builder::{GenerationInput, build_bundle, write_bundle};
    use serde_json::from_str;

    #[test]
    fn parse_answer_boolean_accepts_yes() {
        let question = json!({ "type": "boolean", "required": true });
        assert_eq!(parse_answer(&question, "yes").unwrap(), Value::Bool(true));
    }

    #[test]
    fn parse_answer_integer_handles_numbers() {
        let question = json!({ "type": "integer" });
        assert_eq!(
            parse_answer(&question, "42").unwrap(),
            Value::Number(Number::from(42))
        );
    }

    #[test]
    fn parse_answer_enum_checks_choices() {
        let question = json!({
            "type": "enum",
            "choices": ["alpha", "beta"],
            "required": true
        });
        assert!(parse_answer(&question, "gamma").is_err());
        assert_eq!(
            parse_answer(&question, "alpha").unwrap(),
            Value::String("alpha".into())
        );
    }

    #[test]
    fn parse_answer_respects_defaults() {
        let question = json!({
            "type": "string",
            "default": "default-value",
            "required": true
        });
        assert_eq!(
            parse_answer(&question, "").unwrap(),
            Value::String("default-value".into())
        );
    }

    const FIXTURE: &str = include_str!("../../../ci/fixtures/sample_form_generation.json");

    #[test]
    fn fixture_generates_bundle() {
        let input: GenerationInput =
            from_str(FIXTURE).expect("fixture should deserialize into GenerationInput");
        let bundle = build_bundle(&input).expect("bundle build should succeed");
        let temp_dir = TempDir::new().expect("temp dir");

        let bundle_dir =
            write_bundle(&bundle, &input, temp_dir.path()).expect("bundle write should succeed");

        let forms_dir = bundle_dir.join("forms");
        let flows_dir = bundle_dir.join("flows");
        let examples_dir = bundle_dir.join("examples");
        let schemas_dir = bundle_dir.join("schemas");

        assert!(forms_dir.exists() && forms_dir.join("smoke-form.form.json").exists());
        assert!(flows_dir.exists() && flows_dir.join("smoke-form.qaflow.json").exists());
        assert!(
            examples_dir.exists()
                && examples_dir
                    .join("smoke-form.answers.example.json")
                    .exists()
        );
        assert!(
            schemas_dir.exists() && schemas_dir.join("smoke-form.answers.schema.json").exists()
        );

        let spec_contents =
            fs::read_to_string(forms_dir.join("smoke-form.form.json")).expect("read spec file");
        let spec_value: Value = serde_json::from_str(&spec_contents).expect("spec file JSON");
        assert_eq!(spec_value["id"].as_str(), Some("smoke-form"));
    }
}
