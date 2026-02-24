/// -----------------------------
/// Tokenizer
/// -----------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Number(String), // 数字统一存为字符串，包括科学计数法
    Colon,
    Comma,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Question,
    Lt,
    Gt,
    Enum,
    Equal,
    Pipe,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            '[' => {
                tokens.push(Token::LBracket);
                chars.next();
            }
            ']' => {
                tokens.push(Token::RBracket);
                chars.next();
            }
            '<' => {
                tokens.push(Token::Lt);
                chars.next();
            }
            '>' => {
                tokens.push(Token::Gt);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '?' => {
                tokens.push(Token::Question);
                chars.next();
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            '=' => {
                tokens.push(Token::Equal);
                chars.next();
            }
            '|' => {
                tokens.push(Token::Pipe);
                chars.next();
            }

            // 新逻辑：支持 + / - 开头
            '0'..='9' | '.' | '+' | '-' => {
                let mut num_str = String::new();
                // 如果开头是 + 或 -，先记录并移动
                if let Some(&c) = chars.peek() {
                    if c == '+' || c == '-' {
                        num_str.push(c);
                        chars.next();
                    }
                }

                while let Some(&c) = chars.peek() {
                    // 数字主体部分，包括科学计数法 e/E 和可能的 +/-
                    if
                        c.is_ascii_digit() ||
                        c == '.' ||
                        c == 'e' ||
                        c == 'E' ||
                        c == '+' ||
                        c == '-'
                    {
                        num_str.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                // 尝试解析为 f64 验证格式是否正确
                if num_str.parse::<f64>().is_err() {
                    return Err(format!("Invalid number '{}'", num_str));
                }

                tokens.push(Token::Number(num_str));
            }
            '"' => {
                chars.next(); // skip opening quote
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next(); // skip closing quote
                        break;
                    }
                    // 支持转义字符
                    if c == '\\' {
                        chars.next();
                        if let Some(&esc) = chars.peek() {
                            let esc_ch = match esc {
                                'n' => '\n',
                                'r' => '\r',
                                't' => '\t',
                                '"' => '"',
                                '\\' => '\\',
                                other => other,
                            };
                            s.push(esc_ch);
                            chars.next();
                        }
                    } else {
                        s.push(c);
                        chars.next();
                    }
                }
                tokens.push(Token::Ident(s)); // 字符串作为 Ident 保存
            }
            c if c.is_alphanumeric() || c == '_' => {
                let mut ident = String::new();
                while let Some(&c2) = chars.peek() {
                    if c2.is_alphanumeric() || c2 == '_' {
                        ident.push(c2);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(ident));
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            _ => {
                return Err(format!("Unexpected char '{}'", ch));
            }
        }
    }

    Ok(tokens)
}