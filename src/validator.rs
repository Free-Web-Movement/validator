use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

static REGEX_CACHE: Lazy<Mutex<HashMap<String, Regex>>> = Lazy::new(|| Mutex::new(HashMap::new()));

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
            Self::TypeMismatch {
                field,
                value,
                expected,
                actual,
            } => write!(
                f,
                "{} value {}: expected {}, found {}",
                field, value, expected, actual
            ),
            Self::UnionTypeMismatch {
                field,
                value,
                types,
            } => write!(
                f,
                "{} value {} does not match union types {:?}",
                field, value, types
            ),
            Self::EnumMismatch {
                field,
                value,
                expected,
            } => write!(f, "{} value {} not in enum {:?}", field, value, expected),
            Self::RangeError {
                field,
                value,
                min,
                max,
            } => write!(
                f,
                "{} value {} out of range [{}, {}]",
                field, value, min, max
            ),
            Self::RegexMismatch { field, pattern } => {
                write!(f, "{} regex mismatch: {}", field, pattern)
            }
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
static EMAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").expect("invalid regex"));
static UUID_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[0-9a-fA-F]{8}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{12}$")
        .expect("invalid regex")
});
static IP_V4_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^((25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(25[0-5]|2[0-4]\d|[01]?\d\d?)$")
        .expect("invalid regex")
});
static IP_V6_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$").expect("invalid regex"));
static MAC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$").expect("invalid regex"));
static DATE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").expect("invalid regex"));
static DATETIME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z?$").expect("invalid regex"));
static TIME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\d{2}:\d{2}:\d{2}$").expect("invalid regex"));
static COLOR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^#([0-9a-fA-F]{6}|[0-9a-fA-F]{3})$").expect("invalid regex"));
static HOSTNAME_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:[a-zA-Z0-9_](?:[a-zA-Z0-9_-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,63}$")
        .expect("invalid regex")
});
static SLUG_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").expect("invalid regex"));
static HEX_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[0-9a-fA-F]+$").expect("invalid regex"));
static BASE64_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Za-z0-9+/]+={0,2}$").expect("invalid regex"));
static PHONE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\+?[1-9]\d{1,14}$").expect("invalid regex"));
static CREDITCARD_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[0-9]{13,19}$").expect("invalid regex"));
static ISBN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:ISBN-?1[03]:? )?(?:97[89]-?)?[0-9]{9}[0-9X]$").expect("invalid regex")
});
static PORT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(?:[0-9]{1,4}|[1-5][0-9]{4}|6[0-4][0-9]{3}|65[0-4][0-9]{2}|655[0-2][0-9]|6553[0-5])$",
    )
    .expect("invalid regex")
});
static JSON_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^[\[\]{}:,0-9"'\s-]+$"#).expect("invalid regex"));
static URLENCODED_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9._~-%]+$").expect("invalid regex"));
static SEMVER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(-[0-9a-zA-Z-]+(\.[0-9a-zA-Z-]+)*)?(\+[0-9a-zA-Z-]+(\.[0-9a-zA-Z-]+)*)?$").expect("invalid regex")
});
static USERNAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]{2,19}$").expect("invalid regex"));
static COUNTRYCODE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Z]{2}$").expect("invalid regex"));
static POSTALCODE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Z0-9]{3,10}$").expect("invalid regex"));
static FILEPATH_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:[a-zA-Z]:)?[/\w.-]+$").expect("invalid regex"));
static ALPHA_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z]+$").expect("invalid regex"));
static ALPHANUMERIC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9]+$").expect("invalid regex"));

/// -----------------------------
/// Validator
/// -----------------------------
pub fn validate_field(value: &mut Value, rule: &FieldRule) -> Result<()> {
    // 对对象，先填充默认值
    if let Value::Object(obj) = value
        && !obj.contains_key(&rule.field)
        && let Some(d) = &rule.default
    {
        obj.insert(rule.field.clone(), d.clone());
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

    if !rule.required
        && let Value::String(s) = val
        && s.is_empty()
    {
        return Ok(());
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
    if let Some(enum_vals) = &rule.enum_values
        && !enum_vals.contains(val)
    {
        return Err(ValidationError::EnumMismatch {
            field: rule.field.clone(),
            value: format!("{:?}", val),
            expected: enum_vals.clone(),
        });
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
            let s = val.as_str().ok_or_else(|| {
                ValidationError::Custom(format!("{} not string for regex", field_name))
            })?;
            let re = {
                let mut cache = match REGEX_CACHE.lock() {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::error!("REGEX_CACHE lock poisoned: {}", e);
                        return Err(ValidationError::Custom("regex cache lock poisoned".into()));
                    }
                };
                if let Some(r) = cache.get(pattern) {
                    r.clone()
                } else {
                    match Regex::new(pattern) {
                        Ok(regex) => {
                            cache.insert(pattern.clone(), regex.clone());
                            regex
                        }
                        Err(e) => return Err(ValidationError::InvalidRegex(e.to_string())),
                    }
                }
            };
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

fn validate_range(
    val: &Value,
    min: &Value,
    max: &Value,
    min_inc: bool,
    max_inc: bool,
    field: &str,
) -> Result<()> {
    match val {
        Value::Int(i) => {
            let n = *i as f64;
            let min_v = match min {
                Value::Int(mi) => *mi as f64,
                Value::Float(mf) => mf.ceil(),
                _ => {
                    return Err(ValidationError::Custom(format!(
                        "Invalid min value type for {}",
                        field
                    )));
                }
            };
            let max_v = match max {
                Value::Int(mi) => *mi as f64,
                Value::Float(mf) => mf.floor(),
                _ => {
                    return Err(ValidationError::Custom(format!(
                        "Invalid max value type for {}",
                        field
                    )));
                }
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
                _ => {
                    return Err(ValidationError::Custom(format!(
                        "Invalid min value type in range for {}",
                        field
                    )));
                }
            };
            let max_v = match max {
                Value::Int(mi) => *mi as f64,
                Value::Float(mf) => *mf,
                _ => {
                    return Err(ValidationError::Custom(format!(
                        "Invalid max value type in range for {}",
                        field
                    )));
                }
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
        _ => {
            return Err(ValidationError::Custom(format!(
                "{} cannot apply range constraint to {:?}",
                field, val
            )));
        }
    }
    Ok(())
}

fn parse_usize(val: &Value, field: &str, label: &str) -> Result<usize> {
    match val {
        Value::Int(i) => Ok(*i as usize),
        Value::String(s) => s.parse::<usize>().map_err(|_| {
            ValidationError::Custom(format!(
                "Failed to parse '{}' as usize for {} {}",
                s, field, label
            ))
        }),
        _ => Err(ValidationError::Custom(format!(
            "Invalid {} value type in range for {}",
            label, field
        ))),
    }
}

fn validate_string_type(value: &Value, re: &Regex, type_name: &str) -> Result<()> {
    let s = value.as_str().ok_or(ValidationError::Custom(format!(
        "Not string for {}",
        type_name
    )))?;
    if re.is_match(s) {
        Ok(())
    } else {
        Err(ValidationError::Custom(format!(
            "Invalid {}: {}",
            type_name, s
        )))
    }
}

pub fn validate_type(value: &Value, t: &FieldType) -> Result<()> {
    match t {
        FieldType::String => value
            .as_str()
            .map(|_| ())
            .ok_or(ValidationError::Custom("Not string".into())),
        FieldType::Int => value
            .as_int()
            .map(|_| ())
            .ok_or(ValidationError::Custom("Not int".into())),
        FieldType::Float => value
            .as_float()
            .map(|_| ())
            .ok_or(ValidationError::Custom("Not float".into())),
        FieldType::Bool => value
            .as_bool()
            .map(|_| ())
            .ok_or(ValidationError::Custom("Not bool".into())),
        FieldType::Object => value
            .as_object()
            .map(|_| ())
            .ok_or(ValidationError::Custom("Not object".into())),
        FieldType::Array => value
            .as_array()
            .map(|_| ())
            .ok_or(ValidationError::Custom("Not array".into())),
        FieldType::Email => {
            let s = value
                .as_str()
                .ok_or(ValidationError::Custom("Not string for email".into()))?;
            if !EMAIL_RE.is_match(s) {
                return Err(ValidationError::Custom(format!("Invalid email: {}", s)));
            }
            Ok(())
        }
        FieldType::Uri => {
            let s = value
                .as_str()
                .ok_or(ValidationError::Custom("Not string for uri".into()))?;
            url::Url::parse(s)
                .map(|_| ())
                .map_err(|_| ValidationError::Custom(format!("{} is not a valid URI", s)))
        }
        FieldType::Uuid => {
            let s = value
                .as_str()
                .ok_or(ValidationError::Custom("Not string for uuid".into()))?;
            if !UUID_RE.is_match(s) {
                return Err(ValidationError::Custom(format!("Invalid uuid: {}", s)));
            }
            Ok(())
        }
        FieldType::Ip => {
            let s = value
                .as_str()
                .ok_or(ValidationError::Custom("Not string for ip".into()))?;
            if IP_V4_RE.is_match(s) || IP_V6_RE.is_match(s) {
                Ok(())
            } else {
                Err(ValidationError::Custom(format!("Invalid ip: {}", s)))
            }
        }
        FieldType::Mac => validate_string_type(value, &MAC_RE, "mac"),
        FieldType::Date => validate_string_type(value, &DATE_RE, "date"),
        FieldType::DateTime => validate_string_type(value, &DATETIME_RE, "datetime"),
        FieldType::Time => validate_string_type(value, &TIME_RE, "time"),
        FieldType::Timestamp => value
            .as_int()
            .map(|_| ())
            .ok_or(ValidationError::Custom("Not number for timestamp".into())),
        FieldType::Color => validate_string_type(value, &COLOR_RE, "color"),
        FieldType::Hostname => {
            let s = value
                .as_str()
                .ok_or(ValidationError::Custom("Not string for hostname".into()))?;
            if s.is_empty() || s.len() > 253 {
                return Err(ValidationError::Custom(format!(
                    "Hostname length out of range: {}",
                    s.len()
                )));
            }
            if HOSTNAME_RE.is_match(s) {
                Ok(())
            } else {
                Err(ValidationError::Custom(format!("Invalid hostname: {}", s)))
            }
        }
        FieldType::Slug => validate_string_type(value, &SLUG_RE, "slug"),
        FieldType::Hex => validate_string_type(value, &HEX_RE, "hex"),
        FieldType::Base64 => validate_string_type(value, &BASE64_RE, "base64"),
        FieldType::Password | FieldType::Token => value
            .as_str()
            .map(|_| ())
            .ok_or(ValidationError::Custom(format!("Not string for {:?}", t))),
        FieldType::Phone => validate_string_type(value, &PHONE_RE, "phone"),
        FieldType::CreditCard => {
            let s = value
                .as_str()
                .ok_or(ValidationError::Custom("Not string for creditcard".into()))?;
            if !CREDITCARD_RE.is_match(s) {
                return Err(ValidationError::Custom(format!(
                    "Invalid creditcard: {}",
                    s
                )));
            }
            let digits: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
            if digits.len() < 13 || digits.len() > 19 {
                return Err(ValidationError::Custom(format!(
                    "Invalid creditcard length: {}",
                    digits.len()
                )));
            }
            let mut sum = 0;
            let mut double = false;
            for &d in digits.iter().rev() {
                let mut val = d;
                if double {
                    val = if d > 4 { d * 2 - 9 } else { d * 2 };
                }
                sum += val;
                double = !double;
            }
            if sum % 10 != 0 {
                return Err(ValidationError::Custom(
                    "Invalid creditcard checksum".to_string(),
                ));
            }
            Ok(())
        }
        FieldType::ISBN => validate_string_type(value, &ISBN_RE, "isbn"),
        FieldType::Port => validate_string_type(value, &PORT_RE, "port"),
        FieldType::Json => validate_string_type(value, &JSON_RE, "json"),
        FieldType::UrlEncoded => validate_string_type(value, &URLENCODED_RE, "urlencoded"),
        FieldType::Lat => {
            let s = value
                .as_str()
                .ok_or(ValidationError::Custom("Not string for lat".into()))?;
            if let Ok(f) = s.parse::<f64>() {
                if (-90.0..=90.0).contains(&f) {
                    Ok(())
                } else {
                    Err(ValidationError::Custom(format!(
                        "Latitude out of range: {}",
                        f
                    )))
                }
            } else {
                Err(ValidationError::Custom(format!("Invalid latitude: {}", s)))
            }
        }
        FieldType::Lng => {
            let s = value
                .as_str()
                .ok_or(ValidationError::Custom("Not string for lng".into()))?;
            if let Ok(f) = s.parse::<f64>() {
                if (-180.0..=180.0).contains(&f) {
                    Ok(())
                } else {
                    Err(ValidationError::Custom(format!(
                        "Longitude out of range: {}",
                        f
                    )))
                }
            } else {
                Err(ValidationError::Custom(format!("Invalid longitude: {}", s)))
            }
        }
        FieldType::SemVer => validate_string_type(value, &SEMVER_RE, "semver"),
        FieldType::Username => validate_string_type(value, &USERNAME_RE, "username"),
        FieldType::CountryCode => validate_string_type(value, &COUNTRYCODE_RE, "countrycode"),
        FieldType::PostalCode => validate_string_type(value, &POSTALCODE_RE, "postalcode"),
        FieldType::FilePath => validate_string_type(value, &FILEPATH_RE, "filepath"),
        FieldType::Alpha => validate_string_type(value, &ALPHA_RE, "alpha"),
        FieldType::Alphanumeric => validate_string_type(value, &ALPHANUMERIC_RE, "alphanumeric"),
        FieldType::Custom(pattern) => {
            let s = value.as_str().ok_or(ValidationError::Custom(
                "Not string for custom regex".into(),
            ))?;
            let re = {
                let mut cache = match REGEX_CACHE.lock() {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::error!("REGEX_CACHE lock poisoned: {}", e);
                        return Err(ValidationError::Custom("regex cache lock poisoned".into()));
                    }
                };
                if let Some(r) = cache.get(pattern) {
                    r.clone()
                } else {
                    match Regex::new(pattern) {
                        Ok(regex) => {
                            cache.insert(pattern.clone(), regex.clone());
                            regex
                        }
                        Err(e) => return Err(ValidationError::InvalidRegex(e.to_string())),
                    }
                }
            };
            if re.is_match(s) {
                Ok(())
            } else {
                Err(ValidationError::Custom(format!("Pattern mismatch: {}", s)))
            }
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

pub fn validate(value: &Value, rules: &[FieldRule]) -> Option<Value> {
    let mut validated = value.clone();
    match validate_object(&mut validated, rules) {
        Ok(()) => Some(validated),
        Err(_) => None,
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

fn convert_input_to_value(
    input: &str,
    target_type: &FieldType,
) -> std::result::Result<Value, String> {
    match target_type {
        FieldType::Int => input
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|e| e.to_string()),
        FieldType::Float => input
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|e| e.to_string()),
        FieldType::Bool => match input.to_lowercase().as_str() {
            "true" | "1" => Ok(Value::Bool(true)),
            "false" | "0" => Ok(Value::Bool(false)),
            _ => Err("Invalid boolean value".into()),
        },
        _ => Ok(Value::String(input.to_string())),
    }
}
