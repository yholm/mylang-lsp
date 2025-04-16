use serde::Serialize;

use super::lexer::Token;

#[derive(Serialize, Clone)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: Option<String>,
    pub source: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}
#[derive(Serialize, Clone)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Serialize, Clone)]
#[repr(u8)]
#[allow(dead_code)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

impl Diagnostic {
    pub fn generate(token: &Token, message: &str) -> Self {
        let range = Range {
            start: Position {
                line: token.line as u32,
                character: token.column as u32,
            },
            end: Position {
                line: token.line as u32,
                character: (token.column + token.lexeme.len() - 1) as u32,
            },
        };
        Self {
            range,
            severity: DiagnosticSeverity::Error,
            message: Some(message.to_string()),
            source: Some("custom-lsp".to_string()),
        }
    }
}

impl Default for Range {
    fn default() -> Self {
        let pos = Position {
            line: 0,
            character: 0,
        };

        Self {
            start: pos.clone(),
            end: pos,
        }
    }
}
