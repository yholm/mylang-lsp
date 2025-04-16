pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

#[derive(PartialEq)]
pub enum TokenType {
    PLUS,
    MINUS,
    SLASH,
    STAR,
    CARET,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,

    ARROW,
    PIPE,
    COMMA,
    DOT,
    COLON,
    SEMICOLON,

    EQUAL,
    BANG,
    GREATER,
    LESS,
    EqualEqual,
    BangEqual,
    LessEqual,
    GreaterEqual,

    IDENTIFIER,
    STRING,
    NUMBER,

    TRUE,
    FALSE,
    IF,
    ELSE,
    LET,

    EOF,
}

pub fn lex(source: String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = 0;
    let mut column = 0;
    let mut line = 1;

    while current < source.len() {
        column += 1;
        let start = current;
        let c = source.chars().nth(current).unwrap();

        match c {
            '+' => tokens.push(Token {
                token_type: TokenType::PLUS,
                lexeme: c.to_string(),
                line,
                column,
            }),
            '-' => {
                if match_char(&source, &mut current, '>') {
                    tokens.push(Token {
                        token_type: TokenType::ARROW,
                        lexeme: source[start..=current].to_string(),
                        line,
                        column,
                    });
                } else {
                    tokens.push(Token {
                        token_type: TokenType::MINUS,
                        lexeme: c.to_string(),
                        line,
                        column,
                    });
                }
            }
            '*' => tokens.push(Token {
                token_type: TokenType::STAR,
                lexeme: c.to_string(),
                line,
                column,
            }),
            '/' => {
                if match_char(&source, &mut current, '/') {
                    while current < source.len() && source.chars().nth(current) != Some('\n') {
                        current += 1;
                    }
                } else {
                    tokens.push(Token {
                        token_type: TokenType::SLASH,
                        lexeme: c.to_string(),
                        line,
                        column,
                    });
                }
            }
            '^' => tokens.push(Token {
                token_type: TokenType::CARET,
                lexeme: c.to_string(),
                line,
                column,
            }),
            '(' => tokens.push(Token {
                token_type: TokenType::LeftParen,
                lexeme: c.to_string(),
                line,
                column,
            }),
            ')' => tokens.push(Token {
                token_type: TokenType::RightParen,
                lexeme: c.to_string(),
                line,
                column,
            }),
            '{' => tokens.push(Token {
                token_type: TokenType::LeftBrace,
                lexeme: c.to_string(),
                line,
                column,
            }),
            '}' => tokens.push(Token {
                token_type: TokenType::RightBrace,
                lexeme: c.to_string(),
                line,
                column,
            }),
            '[' => tokens.push(Token {
                token_type: TokenType::LeftBracket,
                lexeme: c.to_string(),
                line,
                column,
            }),
            ']' => tokens.push(Token {
                token_type: TokenType::RightBracket,
                lexeme: c.to_string(),
                line,
                column,
            }),
            '|' => {
                if match_char(&source, &mut current, '>') {
                    tokens.push(Token {
                        token_type: TokenType::PIPE,
                        lexeme: source[start..=current].to_string(),
                        line,
                        column,
                    });
                } else {
                    tokens.push(Token {
                        token_type: TokenType::ARROW,
                        lexeme: c.to_string(),
                        line,
                        column,
                    });
                }
            }
            ',' => tokens.push(Token {
                token_type: TokenType::COMMA,
                lexeme: c.to_string(),
                line,
                column,
            }),
            '.' => tokens.push(Token {
                token_type: TokenType::DOT,
                lexeme: c.to_string(),
                line,
                column,
            }),
            ':' => tokens.push(Token {
                token_type: TokenType::COLON,
                lexeme: c.to_string(),
                line,
                column,
            }),
            ';' => tokens.push(Token {
                token_type: TokenType::SEMICOLON,
                lexeme: c.to_string(),
                line,
                column,
            }),
            '=' => {
                if match_char(&source, &mut current, '=') {
                    tokens.push(Token {
                        token_type: TokenType::EqualEqual,
                        lexeme: source[start..=current].to_string(),
                        line,
                        column,
                    });
                } else {
                    tokens.push(Token {
                        token_type: TokenType::EQUAL,
                        lexeme: c.to_string(),
                        line,
                        column,
                    });
                }
            }
            '!' => {
                if match_char(&source, &mut current, '=') {
                    tokens.push(Token {
                        token_type: TokenType::BangEqual,
                        lexeme: source[start..=current].to_string(),
                        line,
                        column,
                    });
                } else {
                    tokens.push(Token {
                        token_type: TokenType::BANG,
                        lexeme: c.to_string(),
                        line,
                        column,
                    });
                }
            }
            '>' => {
                if match_char(&source, &mut current, '=') {
                    tokens.push(Token {
                        token_type: TokenType::GreaterEqual,
                        lexeme: source[start..=current].to_string(),
                        line,
                        column,
                    });
                } else {
                    tokens.push(Token {
                        token_type: TokenType::GREATER,
                        lexeme: c.to_string(),
                        line,
                        column,
                    });
                }
            }
            '<' => {
                if match_char(&source, &mut current, '=') {
                    tokens.push(Token {
                        token_type: TokenType::LessEqual,
                        lexeme: source[start..=current].to_string(),
                        line,
                        column,
                    });
                } else {
                    tokens.push(Token {
                        token_type: TokenType::LESS,
                        lexeme: c.to_string(),
                        line,
                        column,
                    });
                }
            }
            '0'..='9' => {
                add_number_token(&source, &mut tokens, start, &mut current, &mut column, line);
                continue;
            }
            '"' => {
                add_string_token(
                    &source,
                    &mut tokens,
                    start,
                    &mut current,
                    &mut column,
                    &mut line,
                );
                continue;
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                add_identifier_token(&source, &mut tokens, start, &mut current, &mut column, line);
                continue;
            }
            ' ' | '\r' | '\t' => {
                // Ignore whitespace
            }
            '\n' => {
                line += 1;
                column = 0;
            }
            _ => {
                // Handle unexpected characters
                println!("Unexpected character: {}", c);
            }
        }

        current += 1;
    }

    tokens.push(Token {
        token_type: TokenType::EOF,
        lexeme: String::new(),
        line,
        column,
    });
    tokens
}

fn match_char(source: &String, current: &mut usize, expected: char) -> bool {
    if *current + 1 < source.len() && source.chars().nth(*current + 1).unwrap() == expected {
        *current += 1;
        return true;
    }
    false
}

fn add_number_token(
    source: &String,
    tokens: &mut Vec<Token>,
    start: usize,
    current: &mut usize,
    column: &mut usize,
    line: usize,
) {
    let start_col = *column;

    while *current < source.len() && source.chars().nth(*current).unwrap().is_digit(10) {
        *current += 1;
    }
    let lexeme = &source[start..*current];
    tokens.push(Token {
        token_type: TokenType::NUMBER,
        lexeme: lexeme.to_string(),
        line,
        column: start_col,
    });
}

fn add_string_token(
    source: &String,
    tokens: &mut Vec<Token>,
    start: usize,
    current: &mut usize,
    column: &mut usize,
    line: &mut usize,
) {
    let start_col = *column;

    *current += 1; // Skip the opening quote
    while *current < source.len() && source.chars().nth(*current).unwrap() != '"' {
        if source.chars().nth(*current).unwrap() == '\\' {
            *current += 1;
            *column += 1;
        }
        if source.chars().nth(*current).unwrap() == '\n' {
            *line += 1;
            *column = 0;
        }
        *current += 1;
        *column += 1;
    }
    *current += 1; // Skip the closing quote
    *column += 1;
    let lexeme = &source[start..*current];
    tokens.push(Token {
        token_type: TokenType::STRING,
        lexeme: lexeme.to_string(),
        line: *line,
        column: start_col,
    });
}

fn add_identifier_token(
    source: &String,
    tokens: &mut Vec<Token>,
    start: usize,
    current: &mut usize,
    column: &mut usize,
    line: usize,
) {
    let start_col = *column;

    while *current < source.len()
        && (source.chars().nth(*current).unwrap().is_alphanumeric()
            || source.chars().nth(*current).unwrap() == '_')
    {
        *current += 1;
    }
    let lexeme = &source[start..*current];
    let token_type = match &*lexeme {
        "true" => TokenType::TRUE,
        "false" => TokenType::FALSE,
        "if" => TokenType::IF,
        "else" => TokenType::ELSE,
        "let" => TokenType::LET,
        _ => TokenType::IDENTIFIER,
    };
    tokens.push(Token {
        token_type,
        lexeme: lexeme.to_string(),
        line,
        column: start_col,
    });
}
