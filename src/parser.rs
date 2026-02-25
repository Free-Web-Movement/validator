use crate::{
    ast::{Constraint, Constraints, FieldRule, FieldType, Value},
    token::{Token, tokenize},
};

/// -----------------------------
/// Parser
/// -----------------------------
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }
    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let t = self.next().ok_or("Unexpected EOF")?;
        if &t != expected {
            return Err(format!("Expected {:?}, got {:?}", expected, t));
        }
        Ok(())
    }

    // parse_program 修正版
    pub fn parse_program(&mut self) -> Result<Vec<FieldRule>, String> {
        self.expect(&Token::LParen)?;
        let mut rules = Vec::new();
        loop {
            if matches!(self.peek(), Some(Token::RParen)) {
                self.next();
                break;
            }
            let field = self.parse_field(false)?;
            rules.push(field);

            match self.peek() {
                Some(Token::Comma) => {
                    self.next();
                }
                Some(Token::RParen) => {}
                _ => {
                    return Err("Expected ',' or ')'".into());
                }
            }
        }
        Ok(rules)
    }

    pub fn parse_field(&mut self, nameless: bool) -> Result<FieldRule, String> {
        // -----------------------------
        // 1️⃣ 字段名 + optional
        // -----------------------------
        let (name, optional) = if nameless {
            (String::new(), false)
        } else {
            let name = match self.next() {
                Some(Token::Ident(s)) => s,
                t => {
                    return Err(format!("Expected field name, got {:?}", t));
                }
            };

            let optional = matches!(self.peek(), Some(Token::Question));
            if optional {
                self.next();
            }

            (name, optional)
        };

        if !nameless {
            self.expect(&Token::Colon)?;
        }

        // -----------------------------
        // 2️⃣ 解析 union 类型
        // -----------------------------
        let mut union_types = Vec::new();
        loop {
            let ty = match self.next() {
                Some(Token::Ident(s)) => match s.as_str() {
                    "string" => FieldType::String,
                    "int" => FieldType::Int,
                    "float" => FieldType::Float,
                    "bool" => FieldType::Bool,
                    "object" => FieldType::Object,
                    "array" => FieldType::Array,
                    "email" => FieldType::Email,
                    "uri" => FieldType::Uri,
                    "uuid" => FieldType::Uuid,
                    "ip" => FieldType::Ip,
                    "mac" => FieldType::Mac,
                    "date" => FieldType::Date,
                    "datetime" => FieldType::DateTime,
                    "time" => FieldType::Time,
                    "timestamp" => FieldType::Timestamp,
                    "color" => FieldType::Color,
                    "hostname" => FieldType::Hostname,
                    "slug" => FieldType::Slug,
                    "hex" => FieldType::Hex,
                    "base64" => FieldType::Base64,
                    "password" => FieldType::Password,
                    "token" => FieldType::Token,

                    t => {
                        return Err(format!("Unknown type {}", t));
                    }
                },
                t => {
                    return Err(format!("Expected type, got {:?}", t));
                }
            };

            union_types.push(ty);

            if matches!(self.peek(), Some(Token::Pipe)) {
                self.next();
            } else {
                break;
            }
        }

        let field_type = union_types[0].clone();

        let mut sub_rule = None;
        let mut children = None;
        let mut constraints = Vec::new();
        let mut enum_values = None;
        let mut default = None;
        let is_array = field_type == FieldType::Array;

        //
        // 3️⃣ array<sub_rule>
        //
        // 修正版 array 解析，确保 sub_rule 不被丢弃
        if is_array && matches!(self.peek(), Some(Token::Lt)) {
            self.next(); // consume '<'
            // 使用 nameless=true 避免重复解析字段名，但保留 FieldType、constraints 等
            let sub = self.parse_field(true)?;
            // 父级 array 的 rule 指向这个子规则
            sub_rule = Some(Box::new(FieldRule {
                field: String::new(), // nameless
                field_type: sub.field_type,
                required: sub.required,
                default: sub.default,
                enum_values: sub.enum_values,
                union_types: sub.union_types,
                constraints: sub.constraints,
                rule: sub.rule,
                children: sub.children,
                is_array: sub.is_array,
            }));
            self.expect(&Token::Gt)?;
        }

        //
        // 4️⃣ object(...)
        //
        if field_type == FieldType::Object && matches!(self.peek(), Some(Token::LParen)) {
            self.next(); // consume '('
            let mut inner = Vec::new();

            loop {
                if matches!(self.peek(), Some(Token::RParen)) {
                    self.next(); // consume ')'
                    break;
                }

                inner.push(self.parse_field(false)?);

                match self.peek() {
                    Some(Token::Comma) => {
                        self.next();
                    }
                    Some(Token::RParen) => {}
                    _ => {
                        return Err("Expected ',' or ')' in object".into());
                    }
                }
            }

            children = Some(inner);
        }

        //
        // 5️⃣ 约束 / regex / enum / default
        //
        loop {
            match self.peek() {
                // range
                Some(Token::LBracket) => {
                    constraints.push(self.parse_range(&field_type)?);
                }

                Some(Token::LParen) => {
                    if field_type == FieldType::Object {
                        return Err("Unexpected '(' after object definition".into());
                    }
                    constraints.push(self.parse_range(&field_type)?);
                }

                // regex
                Some(Token::Ident(s)) if s == "regex" => {
                    self.next();
                    self.expect(&Token::LParen)?;
                    let pattern = match self.next() {
                        Some(Token::Ident(p)) => p,
                        t => {
                            return Err(format!("Expected pattern, got {:?}", t));
                        }
                    };
                    self.expect(&Token::RParen)?;
                    constraints.push(Constraint::Regex(pattern));
                }

                // enum
                Some(Token::Ident(s)) if s == "enum" => {
                    self.next();
                    self.expect(&Token::LParen)?;
                    let mut vals = Vec::new();

                    loop {
                        match self.next() {
                            Some(Token::Number(s)) => {
                                // 根据当前的 field_type 转换数字
                                let v = self
                                    .parse_token_number_as_type(&Token::Number(s), &field_type)?;
                                vals.push(v);
                            }
                            Some(Token::Ident(v)) => {
                                // 如果是 bool 类型，特殊处理
                                if field_type == FieldType::Bool {
                                    match v.as_str() {
                                        "true" => vals.push(Value::Bool(true)),
                                        "false" => vals.push(Value::Bool(false)),
                                        _ => vals.push(Value::String(v)),
                                    }
                                } else {
                                    vals.push(Value::String(v));
                                }
                            }
                            t => {
                                return Err(format!("Expected enum value, got {:?}", t));
                            }
                        }

                        match self.peek() {
                            Some(Token::Comma) => {
                                self.next();
                            }
                            Some(Token::RParen) => {
                                self.next();
                                break;
                            }
                            _ => {
                                return Err("Expected ',' or ')' in enum".into());
                            }
                        }
                    }

                    enum_values = Some(vals);
                }

                // default
                Some(Token::Equal) => {
                    self.next();
                    let token = self.next().ok_or("Expected default value")?;

let val = match token {
                        Token::Number(s) => {
                            // 关键修正：如果字段是 string，默认值直接存为 Value::String
                            if field_type == FieldType::String {
                                Value::String(s)
                            } else {
                                // 只有非 string 类型（如 int/float）才走数值解析
                                self.parse_token_number_as_type(&Token::Number(s), &field_type)?
                            }
                        }
                        Token::Ident(s) => {
                            if field_type == FieldType::Bool {
                                match s.as_str() {
                                    "true" => Value::Bool(true),
                                    "false" => Value::Bool(false),
                                    _ => {
                                        return Err(format!("Invalid bool '{}'", s));
                                    }
                                }
                            } else {
                                // 字段是 string 时，Ident 也是字符串
                                Value::String(s)
                            }
                        }
                        t => {
                            return Err(format!("Unexpected default value {:?}", t));
                        }
                    };

                    default = Some(val);
                }

                _ => {
                    break;
                }
            }
        }

        Ok(FieldRule {
            field: name,
            field_type,
            required: if nameless { true } else { !optional },
            default,
            enum_values,
            union_types: if union_types.len() > 1 {
                Some(union_types)
            } else {
                None
            },
            constraints: if constraints.is_empty() {
                None
            } else {
                Some(Constraints { items: constraints })
            },
            rule: sub_rule,
            children,
            is_array,
        })
    }

    /// 根据 FieldType 解析 Token::Number 为 Value
    fn parse_token_number_as_type(
        &self,
        token: &Token,
        field_type: &FieldType,
    ) -> Result<Value, String> {
        match token {
            Token::Number(s) => {
                match field_type {
                    // 如果目标是 Int，允许输入浮点字符串，先解析为 f64
                    FieldType::Int => {
                        if let Ok(i) = s.parse::<i64>() {
                            Ok(Value::Int(i))
                        } else if let Ok(f) = s.parse::<f64>() {
                            // 暂时返回 Float，由 parse_range 进行后续的 ceil/floor 处理
                            Ok(Value::Float(f))
                        } else {
                            Err(format!("Invalid integer '{}'", s))
                        }
                    }
                    FieldType::Float => s
                        .parse::<f64>()
                        .map(Value::Float)
                        .map_err(|_| format!("Invalid float '{}'", s)),
                    // 如果是 String 类型，Range 通常代表长度，所以也支持数字解析
                    FieldType::String => s
                        .parse::<i64>()
                        .map(Value::Int)
                        .map_err(|_| format!("Invalid length number '{}'", s)),
                    _ => Err(format!(
                        "Range only supports int/float/string, got {:?}",
                        field_type
                    )),
                }
            }
            _ => Err("Expected a number token".into()),
        }
    }

    /// Range 解析，支持 int/float 并对 int 类型进行向内取整
    fn parse_range(&mut self, field_type: &FieldType) -> Result<Constraint, String> {
        let min_inclusive = matches!(self.peek(), Some(Token::LBracket));
        self.next(); // 消耗 [ 或 (

        let min_token = self.next().ok_or("Expected min number")?;
        let mut min = self.parse_token_number_as_type(&min_token, field_type)?;

        self.expect(&Token::Comma)?;

        let max_token = self.next().ok_or("Expected max number")?;
        let mut max = self.parse_token_number_as_type(&max_token, field_type)?;

        // --- 逻辑核心：如果字段是 Int，将边界向内取整 ---
        if let FieldType::Int = field_type {
            if let Value::Float(f) = min {
                min = Value::Int(f.ceil() as i64); // 1.2 -> 2
            }
            if let Value::Float(f) = max {
                max = Value::Int(f.floor() as i64); // 5.8 -> 5
            }
        }
        // ------------------------------------------

        let max_inclusive = match self.next() {
            Some(Token::RBracket) => true,
            Some(Token::RParen) => false,
            t => {
                return Err(format!("Expected closing bracket or paren, got {:?}", t));
            }
        };

        Ok(Constraint::Range {
            min,
            max,
            min_inclusive,
            max_inclusive,
        })
    }

    pub fn parse_rules(input: &str) -> Result<Vec<FieldRule>, String> {
        let tokens = tokenize(input)?;
        let mut parser = Parser::new(tokens);
        parser.parse_program()
    }
}
