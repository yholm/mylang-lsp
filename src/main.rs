pub mod lexer;

use std::{
    collections::HashSet,
    io::{self, BufRead, BufReader, Read, Write},
};

use lexer::TokenType;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize)]
struct Diagnostic {
    range: Range,
    severity: DiagnosticSeverity,
    message: Option<String>,
    source: Option<String>,
}

#[derive(Serialize)]
struct Range {
    start: Position,
    end: Position,
}
#[derive(Serialize)]
struct Position {
    line: u32,
    character: u32,
}

#[derive(Serialize)]
#[repr(u8)]
#[allow(dead_code)]
enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

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

fn main() {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut buffer = String::new();

    let mut scope_stack = Vec::new();
    scope_stack.push(generate_globals());

    loop {
        buffer.clear();

        if reader.read_line(&mut buffer).unwrap_or(0) == 0 {
            eprintln!("EOF");
            break;
        }

        let line = buffer.trim();
        if line.starts_with("Content-Length: ") {
            let len = line["Content-Length: ".len()..]
                .trim()
                .parse::<usize>()
                .unwrap();

            buffer.clear();
            if reader.read_line(&mut buffer).unwrap_or(0) == 0 {
                eprintln!("Error: Expected blank line after Content-Length header.");
                break;
            }

            let mut payload = vec![0; len];
            let mut total_read = 0;

            while total_read < len {
                match reader.read(&mut payload[total_read..]) {
                    Ok(0) => {
                        eprintln!("Error: Unexpected EOF while reading payload.");
                        break;
                    }
                    Ok(n) => total_read += n,
                    Err(e) => {
                        eprintln!("Error reading payload: {}", e);
                        break;
                    }
                }
            }

            if total_read != len {
                eprintln!(
                    "Error: Expected {} bytes, but read {} bytes.",
                    len, total_read
                );
                break;
            }

            let message = String::from_utf8(payload).unwrap();
            
            let value = serde_json::from_str::<Value>(&message).unwrap();

            if let Some(method) = value.get("method").and_then(|m| m.as_str()) {

                let param_str = value
                    .get("params")
                    .and_then(|p| p.as_str())
                    .unwrap_or("");

                match method {
                    "textDocument/didOpen" => {
                        let params = serde_json::from_str::<DidOpenParams>(param_str).unwrap();
                        let text = params.text_document.text.clone();
                        let diagnostics = find_unknown_words(text, &mut scope_stack);

                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "textDocument/publishDiagnostics",
                            "params": {
                                "uri": params.text_document.uri,
                                "diagnostics": diagnostics
                            }
                        });
                        let response_str = serde_json::to_string(&response).unwrap();
                        println!("Content-Length: {}\r\n\r\n{}", response_str.len(), response_str);
                        io::stdout().flush().unwrap();
                    }

                    

                    _ => {
                        eprintln!("Unknown method: {}", method);
                    }
                }
            }
        }
    }
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

fn find_unknown_words(text: String, scope_stack: &mut Vec<HashSet<String>>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let tokens = lexer::lex(text);
    let mut i = 0;

    while i < tokens.len() {
        let token = &tokens[i];

        match token.token_type {
            TokenType::LET => {
                i += 1;
                if i >= tokens.len() {
                    let diagnostic =
                        handle_error(token, "Unexpected end of input after 'let' keyword");
                    diagnostics.push(diagnostic);
                    break;
                }

                if tokens[i].token_type != TokenType::IDENTIFIER {
                    let diagnostic = handle_error(
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
                    let diagnostic = handle_error(
                        token,
                        &format!("Duplicate identifier in let statement: {}", lexeme),
                    );
                    diagnostics.push(diagnostic);
                    break;
                }

                scope_stack.last_mut().unwrap().insert(lexeme);
                i += 1;
                if i >= tokens.len() {
                    let diagnostic = handle_error(
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
                            let diagnostic =
                                handle_error(token, &format!("Unknown identifier: {}", lexeme));
                            diagnostics.push(diagnostic);
                        }
                    }
                    i += 1;
                }

                if i > tokens.len() {
                    let diagnostic =
                        handle_error(token, "Unexpected end of input after let statement");
                    diagnostics.push(diagnostic);
                    break;
                }

                scope_stack.pop();
            }

            TokenType::IDENTIFIER => {
                let lexeme = token.lexeme.clone();
                if !scope_stack.iter().any(|set| set.contains(&lexeme)) {
                    let diagnostic =
                        handle_error(token, &format!("Unknown identifier: {}", lexeme));
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
                let diagnostic = handle_error(
                    token,
                    &format!("Duplicate identifier in let statement: {}", token.lexeme),
                );
                diagnostics.push(diagnostic);
            }
        } else if token.token_type == TokenType::ARROW {
            break;
        } else {
            let diagnostic = handle_error(
                token,
                &format!("Unexpected token in let statement: {}", token.lexeme),
            );
            diagnostics.push(diagnostic);
        }

        current += 1;
    }

    added_words
}

fn handle_error(token: &lexer::Token, message: &str) -> Diagnostic {
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
    Diagnostic {
        range,
        severity: DiagnosticSeverity::Error,
        message: Some(message.to_string()),
        source: Some("custom-lsp".to_string()),
    }
}
