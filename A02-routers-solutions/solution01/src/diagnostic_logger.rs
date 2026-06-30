use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::Serialize;

use crate::{model_label_text, PieError};

#[derive(Debug, Clone)]
pub struct DiagnosticLogger {
    log_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum DiagnosticLogLevel {
    Info,
    Error,
}

#[derive(Debug, Serialize)]
struct DiagnosticLogEntry<'a> {
    timestamp: String,
    level: DiagnosticLogLevel,
    event: &'a str,
    command: Option<&'a str>,
    message: String,
    error_code: Option<&'a str>,
    model_label: String,
}

impl DiagnosticLogger {
    pub fn open_at_path(path: impl AsRef<Path>) -> Result<Self, PieError> {
        let log_path = path.as_ref().to_path_buf();
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        Ok(Self { log_path })
    }

    pub fn record_info_event(
        &self,
        event: &str,
        command: Option<&str>,
        message: &str,
    ) -> Result<(), PieError> {
        self.append_log_entry(DiagnosticLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: DiagnosticLogLevel::Info,
            event,
            command,
            message: redact_sensitive_values_from(message),
            error_code: None,
            model_label: model_label_text(),
        })
    }

    pub fn record_error_event(
        &self,
        event: &str,
        command: Option<&str>,
        message: &str,
        error_code: Option<&str>,
    ) -> Result<(), PieError> {
        self.append_log_entry(DiagnosticLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: DiagnosticLogLevel::Error,
            event,
            command,
            message: redact_sensitive_values_from(message),
            error_code,
            model_label: model_label_text(),
        })
    }

    pub fn export_log_text_report(&self) -> Result<String, PieError> {
        let raw_text = std::fs::read_to_string(&self.log_path)?;
        let redacted_text = redact_sensitive_values_from(&raw_text);

        Ok(format!(
            "# PIE Diagnostic Log\n\nPath: {}\n\n{}",
            self.log_path.display(),
            redacted_text
        ))
    }

    fn append_log_entry(&self, entry: DiagnosticLogEntry<'_>) -> Result<(), PieError> {
        let serialized = serde_json::to_string(&entry)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        writeln!(file, "{serialized}")?;
        Ok(())
    }
}

fn redact_sensitive_values_from(input: &str) -> String {
    let mut redacted = String::with_capacity(input.len());
    let mut index = 0;

    while index < input.len() {
        let remaining = &input[index..];

        if remaining.starts_with("Bearer ") || remaining.starts_with("bearer ") {
            let prefix = if remaining.starts_with("Bearer ") {
                "Bearer "
            } else {
                "bearer "
            };
            redacted.push_str(prefix);
            redacted.push_str("[redacted-key]");
            index += prefix.len();
            while index < input.len() && !is_secret_delimiter(input.as_bytes()[index]) {
                index += 1;
            }
            continue;
        }

        if remaining.starts_with("sk-") {
            redacted.push_str("[redacted-key]");
            index += 3;
            while index < input.len() && !is_secret_delimiter(input.as_bytes()[index]) {
                index += 1;
            }
            continue;
        }

        let Some(character) = remaining.chars().next() else {
            break;
        };
        redacted.push(character);
        index += character.len_utf8();
    }

    redacted
}

fn is_secret_delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace()
        || matches!(
            byte,
            b'"' | b'\'' | b',' | b';' | b')' | b'(' | b'[' | b']' | b'{' | b'}'
        )
}
