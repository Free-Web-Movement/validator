use std::collections::HashMap;
use std::fmt;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    ast::{Constraint, FieldRule, FieldType, Value},
    parser::Parser,
    token::tokenize,
};

/// -----------------------------
/// ValidationError
/// -----------------------------
#[derive(Debug, PartialEq, Clone)]
pub enum ValidationError {
    MissingField(String),
    TypeMismatch {
        field: String,
        value: String,
        expected: String,
        actual: String,
    },
    UnionTypeMismatch {
        field: String,
        value: String,
        types: Vec<FieldType>,
    },
    EnumMismatch {
        field: String,
        value: String,
        expected: Vec<Value>,
    },
    RangeError {
        field: String,
        value: String,
        min: String,
        max: String,
    },
    RegexMismatch {
        field: String,
        pattern: String,
    },
    InvalidRegex(String),
    NotAnObject(String),
    Custom(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField(field) => write!(f, "Missing required field {}", field),
            Self::TypeMismatch { field, value, expected, actual } => 
                write!(f, "{} value {}: expected {}, found {}", field, value, expected, actual),
            Self::UnionTypeMismatch { field, value, types } =>
                write!(f, "{} value {} does not match union types {:?}", field, value, types),
            Self::EnumMismatch { field, value, expected } =>
                write!(f, "{} value {} not in enum {:?}", field, value, expected),
            Self::RangeError { field, value, min, max } =>
                write!(f, "{} value {} out of range [{}, {}]", field, value, min, max),
            Self::RegexMismatch { field, pattern } =>
                write!(f, "{} regex mismatch: {}", field, pattern),
            Self::InvalidRegex(err) => write!(f, "Invalid regex: {}", err),
            Self::NotAnObject(field) => write!(f, "{} is not object but has children", field),
            Self::Custom(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for ValidationError {}

pub type Result<T> = std::result::Result<T, ValidationError>;

/// -----------------------------
/// Pre-compiled Regexes
/// -----------------------------
static EMAIL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap());
static UUID_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[0-9a-fA-F]{8}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{12}$").unwrap());
static IP_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^((25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(25[0-5]|2[0-4]\d|[01]?\d\d?)$").unwrap());
static MAC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$").unwrap());
static DATE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap());
static DATETIME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z?$").unwrap());
static TIME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d{2}:\d{2}:\d{2}$").unwrap());
static COLOR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^#([0-9a-fA-F]{6}|[0-9a-fA-F]{3})$").unwrap());
static HOSTNAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?:[a-zA-Z0-9_](?:[a-zA-Z0-9_-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,63}$").unwrap());
static SLUG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap());
static HEX_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[0-9a-fA-F]+$").unwrap());
static BASE64_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z0-9+/]+={0,2}$").unwrap());

/// -----------------------------
/// Validator
/// -----------------------------
pub fn validate_field(value: &mut Value, rule: &FieldRule) -> Result<()> {
    // 对对象，先填充默认值
    if let Value::Object(obj) = value {
        if !obj.contains_key(&rule.field) {
            if let Some(d) = &rule.default {
                obj.insert(rule.field.clone(), d.clone());
            }
        }
    }

    // 获取值
    let val_opt = match value {
        Value::Object(obj) => obj.get_mut(&rule.field),
        _ => Some(value),
    };

    let val = match val_opt {
        Some(v) => v,
        None => {
            if rule.required {
                return Err(ValidationError::MissingField(rule.field.clone()));
            } else {
                return Ok(());
            }
        }
    };

    if !rule.required {
        if let Value::String(s) = val {
            if s.is_empty() {
                return Ok(());
            }
        }
    }

    // union types 验证
    if let Some(types) = &rule.union_types {
        let mut ok = false;
        for t in types {
            if validate_type(val, t).is_ok() {
                ok = true;
                break;
            }
        }
        if !ok {
            return Err(ValidationError::UnionTypeMismatch {
                field: rule.field.clone(),
                value: format!("{:?}", val),
                types: types.clone(),
            });
        }
    } else {
        validate_type(val, &rule.field_type).map_err(|e| {
            if let ValidationError::Custom(msg) = e {
                ValidationError::TypeMismatch {
                    field: rule.field.clone(),
                    value: format!("{:?}", val),
                    expected: format!("{:?}", rule.field_type),
                    actual: msg,
                }
            } else {
                e
            }
        })?;
    }

    // enum 验证
    if let Some(enum_vals) = &rule.enum_values {
        if !enum_vals.contains(val) {
            return Err(ValidationError::EnumMismatch {
                field: rule.field.clone(),
                value: format!("{:?}", val),
                expected: enum_vals.clone(),
            });
        }
    }

    // constraints 验证
    if let Some(c) = &rule.constraints {
        for con in &c.items {
            validate_constraint(val, con, &rule.field)?;
        }
    }

    // sub_rule / array / object 递归验证
    if let Some(sub_rule) = &rule.rule {
        match val {
            Value::Object(_) => validate_field(val, sub_rule)?,
            Value::Array(arr) => {
                for v in arr.iter_mut() {
                    validate_field(v, sub_rule)?;
                }
            }
            _ => {}
        }
    }

    if let Some(children) = &rule.children {
        if let Value::Object(_) = val {
            for child_rule in children {
                validate_field(val, child_rule)?;
            }
        } else {
            return Err(ValidationError::NotAnObject(rule.field.clone()));
        }
    }

    Ok(())
}

fn validate_constraint(val: &Value, con: &Constraint, field_name: &str) -> Result<()> {
    match con {
        Constraint::Range {
            min,
            max,
            min_inclusive,
            max_inclusive,
        } => validate_range(val, min, max, *min_inclusive, *max_inclusive, field_name),
        Constraint::Regex(pattern) => {
            let s = val.as_str().ok_or_else(|| ValidationError::Custom(format!("{} not string for regex", field_name)))?;
            let re = Regex::new(pattern).map_err(|e| ValidationError::InvalidRegex(e.to_string()))?;
            if !re.is_match(s) {
                return Err(ValidationError::RegexMismatch {
                    field: field_name.to_string(),
                    pattern: pattern.clone(),
                });
            }
            Ok(())
        }
    }
}

fn validate_range(val: &Value, min: &Value, max: &Value, min_inc: bool, max_inc: bool, field: &str) -> Result<()> {
    match val {
        Value::Int(i) => {
            let n = *i as f64;
            let min_v = match min {
                Value::Int(mi) => *mi as f64,
                Value::Float(mf) => mf.ceil(),
                _ => return Err(ValidationError::Custom(format!("Invalid min value type for {}", field))),
            };
            let max_v = match max {
                Value::Int(mi) => *mi as f64,
                Value::Float(mf) => mf.floor(),
                _ => return Err(ValidationError::Custom(format!("Invalid max value type for {}", field))),
            };

            let min_ok = if min_inc { n >= min_v } else { n > min_v };
            let max_ok = if max_inc { n <= max_v } else { n < max_v };

            if !min_ok || !max_ok {
                return Err(ValidationError::RangeError {
                    field: field.to_string(),
                    value: i.to_string(),
                    min: min_v.to_string(),
                    max: max_v.to_string(),
                });
            }
        }
        Value::Float(f) => {
            let n = *f;
            let min_v = match min {
                Value::Int(mi) => *mi as f64,
                Value::Float(mf) => *mf,
                _ => return Err(ValidationError::Custom(format!("Invalid min value type in range for {}", field))),
            };
            let max_v = match max {
                Value::Int(mi) => *mi as f64,
                Value::Float(mf) => *mf,
                _ => return Err(ValidationError::Custom(format!("Invalid max value type in range for {}", field))),
            };
            let min_ok = if min_inc { n >= min_v } else { n > min_v };
            let max_ok = if max_inc { n <= max_v } else { n < max_v };
            if !min_ok || !max_ok {
                return Err(ValidationError::RangeError {
                    field: field.to_string(),
                    value: f.to_string(),
                    min: min_v.to_string(),
                    max: max_v.to_string(),
                });
            }
        }
        Value::String(s) => {
            let n = s.len();
            let min_v = parse_usize(min, field, "min")?;
            let max_v = parse_usize(max, field, "max")?;
            let min_ok = if min_inc { n >= min_v } else { n > min_v };
            let max_ok = if max_inc { n <= max_v } else { n < max_v };
            if !min_ok || !max_ok {
                return Err(ValidationError::RangeError {
                    field: field.to_string(),
                    value: n.to_string(),
                    min: min_v.to_string(),
                    max: max_v.to_string(),
                });
            }
        }
        _ => return Err(ValidationError::Custom(format!("{} cannot apply range constraint to {:?}", field, val))),
    }
    Ok(())
}

fn parse_usize(val: &Value, field: &str, label: &str) -> Result<usize> {
    match val {
        Value::Int(i) => Ok(*i as usize),
        Value::String(s) => s.parse::<usize>().map_err(|_| ValidationError::Custom(format!("Failed to parse '{}' as usize for {} {}", s, field, label))),
        _ => Err(ValidationError::Custom(format!("Invalid {} value type in range for {}", label, field))),
    }
}

pub fn validate_type(value: &Value, t: &FieldType) -> Result<()> {
    match t {
        FieldType::String => value.as_str().map(|_| ()).ok_or(ValidationError::Custom("Not string".into())),
        FieldType::Int => value.as_int().map(|_| ()).ok_or(ValidationError::Custom("Not int".into())),
        FieldType::Float => value.as_float().map(|_| ()).ok_or(ValidationError::Custom("Not float".into())),
        FieldType::Bool => value.as_bool().map(|_| ()).ok_or(ValidationError::Custom("Not bool".into())),
        FieldType::Object => value.as_object().map(|_| ()).ok_or(ValidationError::Custom("Not object".into())),
        FieldType::Array => value.as_array().map(|_| ()).ok_or(ValidationError::Custom("Not array".into())),
        FieldType::Email => {
            let s = value.as_str().ok_or(ValidationError::Custom("Not string for email".into()))?;
            if !EMAIL_RE.is_match(s) {
                return Err(ValidationError::Custom(format!("{:?} is not a valid email", value)));
            }
            Ok(())
        }
        FieldType::Uri => {
            let s = value.as_str().ok_or(ValidationError::Custom("Not string for uri".into()))?;
            url::Url::parse(s).map(|_| ()).map_err(|_| ValidationError::Custom(format!("{} is not a valid URI", s)))
        }
        FieldType::Uuid => {
            let s = value.as_str().ok_or(ValidationError::Custom("Not string for uuid".into()))?;
            if !UUID_RE.is_match(s) {
                return Err(ValidationError::Custom(format!("{} is not a valid UUID", s)));
            }
            Ok(())
        }
        FieldType::Ip => {
            let s = value.as_str().ok_or(ValidationError::Custom("Not string for ip".into()))?;
            if IP_RE.is_match(s) { Ok(()) } else { Err(ValidationError::Custom(format!("Invalid ip: {}", s))) }
        }
        FieldType::Mac => {
            let s = value.as_str().ok_or(ValidationError::Custom("Not string for mac".into()))?;
            if MAC_RE.is_match(s) { Ok(()) } else { Err(ValidationError::Custom(format!("Invalid mac: {}", s))) }
        }
        FieldType::Date => {
            let s = value.as_str().ok_or(ValidationError::Custom("Not string for date".into()))?;
            if DATE_RE.is_match(s) { Ok(()) } else { Err(ValidationError::Custom(format!("Invalid date: {}", s))) }
        }
        FieldType::DateTime => {
            let s = value.as_str().ok_or(ValidationError::Custom("Not string for datetime".into()))?;
            if DATETIME_RE.is_match(s) { Ok(()) } else { Err(ValidationError::Custom(format!("Invalid datetime: {}", s))) }
        }
        FieldType::Time => {
            let s = value.as_str().ok_or(ValidationError::Custom("Not string for time".into()))?;
            if TIME_RE.is_match(s) { Ok(()) } else { Err(ValidationError::Custom(format!("Invalid time: {}", s))) }
        }
        FieldType::Timestamp => {
            value.as_int().map(|_| ()).ok_or(ValidationError::Custom("Not number for timestamp".into()))
        }
        FieldType::Color => {
            let s = value.as_str().ok_or(ValidationError::Custom("Not string for color".into()))?;
            if COLOR_RE.is_match(s) { Ok(()) } else { Err(ValidationError::Custom(format!("Invalid color: {}", s))) }
        }
        FieldType::Hostname => {
            let s = value.as_str().ok_or(ValidationError::Custom("Not string for hostname".into()))?;
            if s.is_empty() || s.len() > 253 {
                return Err(ValidationError::Custom(format!("Invalid hostname length: {}", s)));
            }
            if HOSTNAME_RE.is_match(s) { Ok(()) } else { Err(ValidationError::Custom(format!("Invalid hostname: {}", s))) }
        }
        FieldType::Slug => {
            let s = value.as_str().ok_or(ValidationError::Custom("Not string for slug".into()))?;
            if SLUG_RE.is_match(s) { Ok(()) } else { Err(ValidationError::Custom(format!("Invalid slug: {}", s))) }
        }
        FieldType::Hex => {
            let s = value.as_str().ok_or(ValidationError::Custom("Not string for hex".into()))?;
            if HEX_RE.is_match(s) { Ok(()) } else { Err(ValidationError::Custom(format!("Invalid hex: {}", s))) }
        }
        FieldType::Base64 => {
            let s = value.as_str().ok_or(ValidationError::Custom("Not string for base64".into()))?;
            if BASE64_RE.is_match(s) { Ok(()) } else { Err(ValidationError::Custom(format!("Invalid base64: {}", s))) }
        }
        FieldType::Password | FieldType::Token => {
            value.as_str().map(|_| ()).ok_or(ValidationError::Custom(format!("Not string for {:?}", t)))
        }
    }
}

pub fn validate_object(value: &mut Value, rules: &[FieldRule]) -> Result<()> {
    if let Value::Object(_) = value {
        for rule in rules {
            validate_field(value, rule)?;
        }
        Ok(())
    } else {
        Err(ValidationError::Custom("Value is not object".into()))
    }
}

pub fn validate_rule(rule_str: &str, value_str: &str) -> bool {
    let tokens = match tokenize(rule_str) {
        Ok(t) => t,
        Err(_) => return false,
    };

    let mut parser = Parser::new(tokens);
    let rule_ast = match parser.parse_field(false) {
        Ok(r) => r,
        Err(_) => return false,
    };

    let val_enum = match convert_input_to_value(value_str, &rule_ast.field_type) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let mut map = HashMap::new();
    map.insert(rule_ast.field.clone(), val_enum);
    let mut wrapped_value = Value::Object(map);

    validate_field(&mut wrapped_value, &rule_ast).is_ok()
}

fn convert_input_to_value(input: &str, target_type: &FieldType) -> std::result::Result<Value, String> {
    match target_type {
        FieldType::Int => input.parse::<i64>().map(Value::Int).map_err(|e| e.to_string()),
        FieldType::Float => input.parse::<f64>().map(Value::Float).map_err(|e| e.to_string()),
        FieldType::Bool => match input.to_lowercase().as_str() {
            "true" | "1" => Ok(Value::Bool(true)),
            "false" | "0" => Ok(Value::Bool(false)),
            _ => Err("Invalid boolean value".into()),
        },
        _ => Ok(Value::String(input.to_string())),
    }
}
