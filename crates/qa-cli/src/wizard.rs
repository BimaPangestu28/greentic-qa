use serde_json::Value;

/// Controls which bits of state the wizard prints.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Verbosity {
    /// Clean output: question prompts only.
    Clean,
    /// Debug output: status, visible questions, error details, help text.
    Debug,
}

impl Verbosity {
    pub fn from_debug(debug: bool) -> Self {
        if debug {
            Verbosity::Debug
        } else {
            Verbosity::Clean
        }
    }

    pub fn is_debug(&self) -> bool {
        matches!(self, Verbosity::Debug)
    }
}

/// Toolbar responsible for printing prompts once the engine yields a question.
pub struct WizardPresenter {
    verbosity: Verbosity,
    header_printed: bool,
}

impl WizardPresenter {
    pub fn new(verbosity: Verbosity) -> Self {
        Self {
            verbosity,
            header_printed: false,
        }
    }

    pub fn show_header(&mut self, payload: &WizardPayload) {
        if self.header_printed {
            return;
        }
        println!("Form: {}", payload.form_title);
        if self.verbosity.is_debug() {
            if let Some(help) = &payload.help {
                println!("Help: {}", help);
            }
        }
        self.header_printed = true;
    }

    pub fn show_status(&self, payload: &WizardPayload) {
        if self.verbosity.is_debug() {
            println!(
                "Status: {} ({}/{})",
                payload.status.as_str(),
                payload.progress.answered,
                payload.progress.total
            );
            self.print_visible_questions(payload);
        } else if payload.status == RenderStatus::NeedInput && payload.visible_count() == 0 {
            println!("No visible questions are available; check your conditional logic.");
        }
    }

    fn print_visible_questions(&self, payload: &WizardPayload) {
        println!("Visible questions:");
        for question in payload.questions.iter().filter(|question| question.visible) {
            let mut entry = format!(" - {} ({})", question.id, question.title);
            if question.required {
                entry.push_str(" [required]");
            }
            println!("{}", entry);
        }
    }

    pub fn show_prompt(&self, prompt: &PromptContext) {
        let mut line = if prompt.total > 0 {
            format!("{}/{} {}", prompt.index, prompt.total, prompt.title)
        } else {
            format!("{} {}", prompt.index, prompt.title)
        };
        if prompt.required {
            line.push('*');
        }
        if let Some(hint) = &prompt.hint {
            line.push(' ');
            line.push_str(hint);
        }
        println!("{}", line);
        if let Some(description) = &prompt.description {
            println!("{}", description);
        }
        if self.verbosity.is_debug() && !prompt.choices.is_empty() {
            println!("Choices: {}", prompt.choices.join(", "));
        }
    }

    pub fn show_parse_error(&self, error: &AnswerParseError) {
        eprintln!("Invalid answer: {}", error.user_message);
        if self.verbosity.is_debug() {
            if let Some(debug) = &error.debug_message {
                eprintln!("  Debug: {}", debug);
            }
        }
    }

    pub fn show_completion(&self, answers: &Value) {
        println!("Done ✅");
        match serde_json::to_string_pretty(answers) {
            Ok(pretty) => println!("{}", pretty),
            Err(_) => println!("{}", answers),
        }
    }
}

/// Render payload extracted from the component output.
pub struct WizardPayload {
    pub form_title: String,
    pub help: Option<String>,
    pub status: RenderStatus,
    pub progress: RenderProgress,
    pub questions: Vec<WizardQuestion>,
}

impl WizardPayload {
    pub fn from_json(json: &Value) -> Result<Self, String> {
        let form_title = json
            .get("form_title")
            .and_then(Value::as_str)
            .ok_or_else(|| "wizard payload missing form_title".to_string())?
            .to_string();
        let help = json
            .get("help")
            .and_then(Value::as_str)
            .map(|value| value.to_string());
        let status_str = json
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("need_input");
        let status = RenderStatus::from_label(status_str);
        let progress = json
            .get("progress")
            .and_then(Value::as_object)
            .ok_or_else(|| "wizard payload missing progress".to_string())?;
        let answered = progress
            .get("answered")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        let total = progress.get("total").and_then(Value::as_u64).unwrap_or(0) as usize;
        let questions = json
            .get("questions")
            .and_then(Value::as_array)
            .ok_or_else(|| "wizard payload missing questions".to_string())?
            .iter()
            .map(WizardQuestion::from_json)
            .collect::<Result<_, _>>()?;
        Ok(Self {
            form_title,
            help,
            status,
            progress: RenderProgress { answered, total },
            questions,
        })
    }

    pub fn visible_count(&self) -> usize {
        self.questions
            .iter()
            .filter(|question| question.visible)
            .count()
    }

    pub fn question(&self, id: &str) -> Option<&WizardQuestion> {
        self.questions.iter().find(|question| question.id == id)
    }
}

/// Progress counters from the render payload.
pub struct RenderProgress {
    pub answered: usize,
    pub total: usize,
}

/// Status returned by the renderer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RenderStatus {
    NeedInput,
    Complete,
    Error,
}

impl RenderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RenderStatus::NeedInput => "need_input",
            RenderStatus::Complete => "complete",
            RenderStatus::Error => "error",
        }
    }

    pub fn from_label(label: &str) -> Self {
        match label {
            "complete" => RenderStatus::Complete,
            "error" => RenderStatus::Error,
            _ => RenderStatus::NeedInput,
        }
    }
}

/// Minimal view of a question used for rendering prompts.
pub struct WizardQuestion {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub kind: QuestionKind,
    pub required: bool,
    pub choices: Vec<String>,
    pub visible: bool,
}

impl WizardQuestion {
    fn from_json(value: &Value) -> Result<Self, String> {
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "question missing id".to_string())?
            .to_string();
        let title = value
            .get("title")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("question '{}' missing title", id))?
            .to_string();
        let description = value
            .get("description")
            .and_then(Value::as_str)
            .map(|value| value.to_string());
        let required = value
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let kind_label = value
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("string");
        let kind = QuestionKind::from_label(kind_label);
        let choices = value
            .get("choices")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let visible = value
            .get("visible")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        Ok(Self {
            id,
            title,
            description,
            kind,
            required,
            choices,
            visible,
        })
    }
}

/// Context used to format a single prompt.
pub struct PromptContext {
    pub index: usize,
    pub total: usize,
    pub title: String,
    pub description: Option<String>,
    pub required: bool,
    pub hint: Option<String>,
    pub choices: Vec<String>,
}

impl PromptContext {
    pub fn new(question: &WizardQuestion, progress: &RenderProgress) -> Self {
        let index = progress.answered + 1;
        let total = progress.total;
        let hint = question.kind.hint(&question.choices);
        Self {
            index: index.max(1),
            total,
            title: question.title.clone(),
            description: question.description.clone(),
            required: question.required,
            hint,
            choices: question.choices.clone(),
        }
    }
}

/// Supported kinds for question prompts.
#[derive(Copy, Clone)]
pub enum QuestionKind {
    String,
    Boolean,
    Integer,
    Number,
    Enum,
    Unknown,
}

impl QuestionKind {
    fn from_label(label: &str) -> Self {
        match label {
            "string" => QuestionKind::String,
            "boolean" => QuestionKind::Boolean,
            "integer" => QuestionKind::Integer,
            "number" => QuestionKind::Number,
            "enum" => QuestionKind::Enum,
            _ => QuestionKind::Unknown,
        }
    }

    fn hint(&self, choices: &[String]) -> Option<String> {
        match self {
            QuestionKind::Boolean => Some("(yes/no)".to_string()),
            QuestionKind::Integer => Some("(integer)".to_string()),
            QuestionKind::Number => Some("(number)".to_string()),
            QuestionKind::Enum if !choices.is_empty() => Some(format!("({})", choices.join("/"))),
            _ => None,
        }
    }
}

/// Error produced when parsing answers from the user.
#[derive(Debug)]
pub struct AnswerParseError {
    pub user_message: String,
    pub debug_message: Option<String>,
}

impl AnswerParseError {
    pub fn new(user_message: impl Into<String>, debug_message: Option<String>) -> Self {
        Self {
            user_message: user_message.into(),
            debug_message,
        }
    }
}
