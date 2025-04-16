pub mod diagnostics;
pub mod lexer;
use diagnostics::{Diagnostic, DiagnosticSeverity, Range};
use lexer::TokenType;
use std::collections::HashSet;

use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct DidOpenParams {
    #[serde(rename = "textDocument")]
    text_document: TextDocument,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct InitializeParams {
    capabilities: Value,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct TextDocument {
    uri: String,
    #[serde(rename = "languageId")]
    language_id: String,
    version: u32,
    text: String,
}

pub fn run_analysis(message: String) -> Result<String, Diagnostic> {
    let value = serde_json::from_str::<Value>(&message).map_err(|e| Diagnostic {
        range: Range::default(),
        severity: DiagnosticSeverity::Error,
        message: Some(format!("Invalid JSON: {}", e)),
        source: Some("custom-lsp".to_string()),
    })?;

    let method = value
        .get("method")
        .and_then(|m| m.as_str())
        .ok_or_else(|| Diagnostic {
            range: Range::default(),
            severity: DiagnosticSeverity::Error,
            message: Some("Missing 'method' field".to_string()),
            source: Some("custom-lsp".to_string()),
        })?;

    let params = value.get("params").ok_or_else(|| Diagnostic {
        range: Range::default(),
        severity: DiagnosticSeverity::Error,
        message: Some("Missing 'params' field".to_string()),
        source: Some("custom-lsp".to_string()),
    })?;

    let mut diagnostics = Vec::new();
    let mut scope_stack = Vec::new();
    scope_stack.push(generate_globals());

    let mut response = json!(null);

    match method {
        "initialize" => {
            let param: InitializeParams =
                serde_json::from_value(params.clone()).map_err(|e| Diagnostic {
                    range: Range::default(),
                    severity: DiagnosticSeverity::Error,
                    message: Some(format!("Invalid initialize params: {}", e)),
                    source: Some("custom-lsp".to_string()),
                })?;
        }

        "textDocument/didOpen" => {
            let param: DidOpenParams =
                serde_json::from_value(params.clone()).map_err(|e| Diagnostic {
                    range: Range::default(),
                    severity: DiagnosticSeverity::Error,
                    message: Some(format!("Invalid didOpen params: {}", e)),
                    source: Some("custom-lsp".to_string()),
                })?;

            let text = param.text_document.text;
            let word_errors = find_unknown_words(&text, &mut scope_stack);
            diagnostics.extend(word_errors);

            response = json!({
                "jsonrpc": "2.0",
                "method": "textDocument/publishDiagnostics",
                "params": {
                    "uri": param.text_document.uri,
                    "diagnostics": diagnostics
                }
            });
        }

        _ => {}
    };

    let output = serde_json::to_string(&response).unwrap();
    Ok(output)
}

fn generate_globals() -> HashSet<String> {
    let mut known_words = HashSet::new();
    known_words.insert("let".to_string());
    known_words.insert("if".to_string());
    known_words.insert("else".to_string());
    known_words.insert("true".to_string());
    known_words.insert("false".to_string());

    known_words
}

fn find_unknown_words(text: &String, scope_stack: &mut Vec<HashSet<String>>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let tokens = lexer::lex(text.to_string());
    let mut i = 0;

    while i < tokens.len() {
        let token = &tokens[i];

        match token.token_type {
            TokenType::LET => {
                i += 1;
                if i >= tokens.len() {
                    let diagnostic = Diagnostic::generate(token, "Unexpected termination");
                    diagnostics.push(diagnostic);
                    break;
                }

                if tokens[i].token_type != TokenType::IDENTIFIER {
                    let diagnostic = Diagnostic::generate(
                        token,
                        &format!(
                            "Expected identifier after 'let', found: {}",
                            tokens[i].lexeme
                        ),
                    );
                    diagnostics.push(diagnostic);
                    break;
                }

                let lexeme = tokens[i].lexeme.clone();
                if scope_stack.last().unwrap().contains(&lexeme) {
                    let diagnostic = Diagnostic::generate(
                        token,
                        &format!("Duplicate identifier in let statement: {}", lexeme),
                    );
                    diagnostics.push(diagnostic);
                    break;
                }

                scope_stack.last_mut().unwrap().insert(lexeme);
                i += 1;
                if i >= tokens.len() {
                    let diagnostic = Diagnostic::generate(
                        token,
                        "Unexpected end of input after identifier in let statement",
                    );
                    diagnostics.push(diagnostic);
                    break;
                }

                let added_words = handle_let_statement(&tokens[i..], &mut diagnostics);
                scope_stack.push(added_words);

                while tokens[i].token_type != TokenType::SEMICOLON {
                    if tokens[i].token_type == TokenType::IDENTIFIER {
                        let lexeme = tokens[i].lexeme.clone();
                        if !scope_stack.last().unwrap().contains(&lexeme) {
                            let diagnostic = Diagnostic::generate(
                                &tokens[i],
                                &format!("Unknown identifier: {}", lexeme),
                            );
                            diagnostics.push(diagnostic);
                        }
                    }
                    i += 1;
                }

                if i > tokens.len() {
                    let diagnostic =
                        Diagnostic::generate(token, "Unexpected end of input after let statement");
                    diagnostics.push(diagnostic);
                    break;
                }

                scope_stack.pop();
            }

            TokenType::IDENTIFIER => {
                let lexeme = token.lexeme.clone();
                if !scope_stack.iter().any(|set| set.contains(&lexeme)) {
                    let diagnostic =
                        Diagnostic::generate(token, &format!("Unknown identifier: {}", lexeme));
                    diagnostics.push(diagnostic);
                }
            }

            _ => {
                // Handle other token types if necessary
            }
        }

        i += 1;
    }

    diagnostics
}

fn handle_let_statement(
    tokens: &[lexer::Token],
    diagnostics: &mut Vec<Diagnostic>,
) -> HashSet<String> {
    let mut current = 0;

    let mut added_words = HashSet::new();

    while current < tokens.len() {
        let token = &tokens[current];

        if token.token_type == TokenType::IDENTIFIER {
            let lexeme = token.lexeme.clone();
            if !added_words.contains(&lexeme) {
                added_words.insert(lexeme);
            } else {
                let diagnostic = Diagnostic::generate(
                    token,
                    &format!("Duplicate identifier in let statement: {}", token.lexeme),
                );
                diagnostics.push(diagnostic);
            }
        } else if token.token_type == TokenType::ARROW {
            break;
        } else {
            let diagnostic = Diagnostic::generate(
                token,
                &format!("Unexpected token in let statement: {}", token.lexeme),
            );
            diagnostics.push(diagnostic);
        }

        current += 1;
    }

    added_words
}
