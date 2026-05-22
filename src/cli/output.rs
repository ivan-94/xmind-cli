use std::ffi::OsString;

use clap::error::ErrorKind;
use serde::Serialize;
use serde_json::{json, Value};

pub fn render_parse_error(args: &[OsString], error: clap::Error) -> i32 {
    match error.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
            print!("{}", error.render());
            0
        }
        _ if args_include_json(args) => {
            let error = CliErrorBody::new(
                ErrorCode::InvalidUsage,
                error.to_string(),
                true,
                "Correct the command arguments and retry.",
            )
            .with_details(json!({
                "clap_kind": format!("{:?}", error.kind()),
            }));

            let exit_code = error.exit_code;
            let envelope = CommandEnvelope::<Value> {
                ok: false,
                command: infer_command(args),
                workbook: infer_workbook(args),
                dry_run: false,
                applied: false,
                result: None,
                error: Some(CliErrorBody { exit_code, ..error }),
                warnings: Vec::new(),
            };

            render_json_envelope(&envelope);
            exit_code
        }
        _ => {
            eprint!("{}", error.render());
            error.exit_code()
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CommandEnvelope<T>
where
    T: Serialize,
{
    pub ok: bool,

    pub command: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub workbook: Option<String>,

    pub dry_run: bool,
    pub applied: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CliErrorBody>,

    pub warnings: Vec<CliWarning>,
}

#[derive(Debug, Serialize)]
pub struct CliErrorBody {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
    pub suggested_fix: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<CandidateDto>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_index: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_path: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    pub exit_code: i32,

    #[serde(skip_serializing_if = "Value::is_null")]
    pub details: Value,
}

impl CliErrorBody {
    pub fn new(
        code: ErrorCode,
        message: impl Into<String>,
        retryable: bool,
        suggested_fix: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            retryable,
            suggested_fix: suggested_fix.into(),
            selector: None,
            candidates: Vec::new(),
            operation_index: None,
            operation: None,
            field_path: None,
            path: None,
            exit_code: code.exit_code(),
            details: Value::Null,
        }
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn with_selector(mut self, selector: impl Into<String>) -> Self {
        self.selector = Some(selector.into());
        self
    }

    pub fn with_candidates(mut self, candidates: Vec<CandidateDto>) -> Self {
        self.candidates = candidates;
        self
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = details;
        self
    }

    pub fn with_field_path(mut self, field_path: impl Into<String>) -> Self {
        self.field_path = Some(field_path.into());
        self
    }

    pub fn with_operation_context(
        mut self,
        operation_index: usize,
        operation: impl Into<String>,
    ) -> Self {
        self.operation_index = Some(operation_index);
        self.operation = Some(operation.into());
        self
    }
}

#[derive(Debug, Serialize)]
pub struct CliWarning {
    pub code: String,
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CandidateDto {
    pub id: String,
    pub path: String,
    pub title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sheet: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidUsage,
    FileNotFound,
    ParseFailed,
    UnsupportedFormat,
    SheetNotFound,
    AmbiguousSheet,
    NotFound,
    AmbiguousSelector,
    InvalidTreeInput,
    InvalidPatch,
    PatchConflict,
    ValidationFailed,
    WriteFailed,
    UnsupportedAssetType,
    RootOperationNotAllowed,
}

impl ErrorCode {
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::InvalidUsage => 2,
            Self::FileNotFound => 3,
            Self::ParseFailed => 4,
            Self::UnsupportedFormat => 11,
            Self::SheetNotFound | Self::NotFound => 5,
            Self::AmbiguousSheet | Self::AmbiguousSelector => 6,
            Self::InvalidTreeInput | Self::InvalidPatch => 7,
            Self::PatchConflict | Self::RootOperationNotAllowed => 8,
            Self::ValidationFailed => 9,
            Self::WriteFailed => 10,
            Self::UnsupportedAssetType => 11,
        }
    }
}

pub fn render_json_envelope<T>(envelope: &CommandEnvelope<T>)
where
    T: Serialize,
{
    println!(
        "{}",
        serde_json::to_string_pretty(envelope).expect("command envelope serializes")
    );
}

pub fn render_human_error(command: Option<&str>, error: &CliErrorBody, no_color: bool) {
    match (command, no_color) {
        (Some(command), true) => eprintln!("{command}: {}", error.message),
        (Some(command), false) => eprintln!("\u{1b}[31m{command}\u{1b}[0m: {}", error.message),
        (None, _) => eprintln!("{}", error.message),
    }
}

fn args_include_json(args: &[OsString]) -> bool {
    args.iter().any(|arg| arg == "--json")
}

fn infer_command(args: &[OsString]) -> Option<String> {
    args.iter()
        .skip(1)
        .find_map(|arg| arg.to_str().filter(|value| !value.starts_with('-')))
        .map(ToOwned::to_owned)
}

fn infer_workbook(args: &[OsString]) -> Option<String> {
    let mut seen_command = false;

    for arg in args.iter().skip(1).filter_map(|arg| arg.to_str()) {
        if arg.starts_with('-') {
            continue;
        }

        if seen_command {
            return Some(arg.to_owned());
        }

        seen_command = true;
    }

    None
}
