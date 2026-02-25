use std::collections::HashMap;

use crate::{
    ast::{Constraint, FieldRule, FieldType, Value},
    parser::Parser,
    token::tokenize,
};
use regex::Regex;

/// -----------------------------
/// Validator
/// -----------------------------
pub fn validate_field(value: &mut Value, rule: &FieldRule) -> Result<(), String> {
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

    if val_opt.is_none() {
        if rule.required {
            return Err(format!("Missing required field {}", rule.field));
        } else {
            return Ok(());
        }
    }

    let val = val_opt.unwrap();
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
            return Err(format!(
                "{} value {:?} does not match union types {:?}",
                rule.field, val, types
            ));
        }
    } else {
        // validate_type(val, &rule.field_type)?;
        validate_type(val, &rule.field_type)
            .map_err(|e| format!("{} value {:?}: {}", rule.field, val, e))?;
    }

    // enum 验证
    if let Some(enum_vals) = &rule.enum_values {
        if !enum_vals.contains(val) {
            return Err(format!(
                "{} value {:?} not in enum {:?}",
                rule.field, val, enum_vals
            ));
        }
    }

    // constraints 验证
    if let Some(c) = &rule.constraints {
        for con in &c.items {
            match con {
                Constraint::Range {
                    min,
                    max,
                    min_inclusive,
                    max_inclusive,
                } => {
                    match val {
                        Value::Int(i) => {
                            let n = *i as f64;
                            // --- 逻辑修正：最小值向上取整，最大值向下取整 ---
                            let min_v = match min {
                                Value::Int(mi) => *mi as f64,
                                Value::Float(mf) => mf.ceil(), // 1.2 -> 2.0
                                _ => {
                                    return Err(format!(
                                        "Invalid min value type for {}",
                                        rule.field
                                    ));
                                }
                            };
                            let max_v = match max {
                                Value::Int(mi) => *mi as f64,
                                Value::Float(mf) => mf.floor(), // 5.8 -> 5.0
                                _ => {
                                    return Err(format!(
                                        "Invalid max value type for {}",
                                        rule.field
                                    ));
                                }
                            };
                            // ------------------------------------------

                            let min_ok = if *min_inclusive {
                                n >= min_v
                            } else {
                                n > min_v
                            };
                            let max_ok = if *max_inclusive {
                                n <= max_v
                            } else {
                                n < max_v
                            };

                            if !min_ok || !max_ok {
                                return Err(format!(
                                    "{} value {} out of range [{}, {}]",
                                    rule.field, i, min_v, max_v
                                ));
                            }
                        }
                        Value::Float(f) => {
                            let n = *f;
                            let min_v = match min {
                                Value::Int(mi) => *mi as f64,
                                Value::Float(mf) => *mf,
                                _ => {
                                    return Err(format!(
                                        "Invalid min value type in range for {}",
                                        rule.field
                                    ));
                                }
                            };
                            let max_v = match max {
                                Value::Int(mi) => *mi as f64,
                                Value::Float(mf) => *mf,
                                _ => {
                                    return Err(format!(
                                        "Invalid max value type in range for {}",
                                        rule.field
                                    ));
                                }
                            };
                            let min_ok = if *min_inclusive {
                                n >= min_v
                            } else {
                                n > min_v
                            };
                            let max_ok = if *max_inclusive {
                                n <= max_v
                            } else {
                                n < max_v
                            };
                            if !min_ok || !max_ok {
                                return Err(format!(
                                    "{} value {:?} out of range [{:?}, {:?}]",
                                    rule.field, val, min, max
                                ));
                            }
                        }
                        Value::String(s) => {
                            let n = s.len();
                            // min/max 可以是 Value::Int 或 Value::String
                            let min_v = match min {
                                Value::Int(mi) => *mi as usize,
                                Value::String(s) => s
                                    .parse::<usize>()
                                    .map_err(|_| format!("Failed to parse '{}' as usize", s))?,
                                _ => {
                                    return Err(format!(
                                        "Invalid min value type in range for {}",
                                        rule.field
                                    ));
                                }
                            };
                            let max_v = match max {
                                Value::Int(mi) => *mi as usize,
                                Value::String(s) => s
                                    .parse::<usize>()
                                    .map_err(|_| format!("Failed to parse '{}' as usize", s))?,
                                _ => {
                                    return Err(format!(
                                        "Invalid max value type in range for {}",
                                        rule.field
                                    ));
                                }
                            };
                            let min_ok = if *min_inclusive {
                                n >= min_v
                            } else {
                                n > min_v
                            };
                            let max_ok = if *max_inclusive {
                                n <= max_v
                            } else {
                                n < max_v
                            };
                            if !min_ok || !max_ok {
                                return Err(format!(
                                    "{} length {} out of range [{:?}, {:?}]",
                                    rule.field, n, min, max
                                ));
                            }
                        }
                        _ => {
                            return Err(format!(
                                "{} cannot apply range constraint to {:?}",
                                rule.field, val
                            ));
                        }
                    }
                }
                Constraint::Regex(pattern) => {
                    let s = val
                        .as_str()
                        .ok_or(format!("{} not string for regex", rule.field))?;
                    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex: {}", e))?;
                    if !re.is_match(s) {
                        return Err(format!("{} regex mismatch: {}", rule.field, pattern));
                    }
                }
            }
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
            return Err(format!("{} is not object but has children", rule.field));
        }
    }

    Ok(())
}

pub fn validate_type(value: &Value, t: &FieldType) -> Result<(), String> {
    match t {
        FieldType::String => {
            if value.as_str().is_some() {
                Ok(())
            } else {
                Err("Not string".into())
            }
        }
        FieldType::Int => {
            if value.as_int().is_some() {
                Ok(())
            } else {
                Err("Not int".into())
            }
        }
        FieldType::Float => {
            if value.as_float().is_some() {
                Ok(())
            } else {
                Err("Not float".into())
            }
        }
        FieldType::Bool => {
            if value.as_bool().is_some() {
                Ok(())
            } else {
                Err("Not bool".into())
            }
        }
        FieldType::Object => {
            if value.as_object().is_some() {
                Ok(())
            } else {
                Err("Not object".into())
            }
        }
        FieldType::Array => {
            if value.as_array().is_some() {
                Ok(())
            } else {
                Err("Not array".into())
            }
        }
        FieldType::Email => {
            let s = value.as_str().ok_or("Not string for email")?;
            let re = Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();
            if !re.is_match(s) {
                return Err(format!("{:?} is not a valid email", value));
            }
            Ok(())
        }
        FieldType::Uri => {
            let s = value.as_str().ok_or("Not string for uri")?;
            let url = url::Url::parse(s).map_err(|_| format!("{} is not a valid URI", s))?;
            Ok(())
        }
        FieldType::Uuid => {
            let s = value.as_str().ok_or("Not string for uuid")?;
            let re = Regex::new(
                r"^[0-9a-fA-F]{8}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{12}$"
            ).unwrap();
            if !re.is_match(s) {
                return Err(format!("{} is not a valid UUID", s));
            }
            Ok(())
        }

        FieldType::Ip => {
            let s = value.as_str().ok_or("Not string for ip")?;
            let re =
                Regex::new(r"^((25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(25[0-5]|2[0-4]\d|[01]?\d\d?)$")
                    .unwrap();
            if re.is_match(s) {
                Ok(())
            } else {
                Err(format!("Invalid ip: {}", s))
            }
        }
        FieldType::Mac => {
            let s = value.as_str().ok_or("Not string for mac")?;
            let re = Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$").unwrap();
            if re.is_match(s) {
                Ok(())
            } else {
                Err(format!("Invalid mac: {}", s))
            }
        }
        FieldType::Date => {
            let s = value.as_str().ok_or("Not string for date")?;
            let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
            if re.is_match(s) {
                Ok(())
            } else {
                Err(format!("Invalid date: {}", s))
            }
        }
        FieldType::DateTime => {
            let s = value.as_str().ok_or("Not string for datetime")?;
            let re = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z?$").unwrap();
            if re.is_match(s) {
                Ok(())
            } else {
                Err(format!("Invalid datetime: {}", s))
            }
        }
        FieldType::Time => {
            let s = value.as_str().ok_or("Not string for time")?;
            let re = Regex::new(r"^\d{2}:\d{2}:\d{2}$").unwrap();
            if re.is_match(s) {
                Ok(())
            } else {
                Err(format!("Invalid time: {}", s))
            }
        }
        FieldType::Timestamp => {
            value.as_int().ok_or("Not number for timestamp")?;
            Ok(())
        }
        FieldType::Color => {
            let s = value.as_str().ok_or("Not string for color")?;
            let re = Regex::new(r"^#([0-9a-fA-F]{6}|[0-9a-fA-F]{3})$").unwrap();
            if re.is_match(s) {
                Ok(())
            } else {
                Err(format!("Invalid color: {}", s))
            }
        }
        FieldType::Hostname => {
            let s = value.as_str().ok_or("Not string for hostname")?;
            // 1. 手动检查总长度 (对应原正则中的 (?=.{1,253}$) )
            if s.is_empty() || s.len() > 253 {
                return Err(format!("Invalid hostname length: {}", s));
            }
            // 2. 使用不含断言的兼容正则
            let re = Regex::new(
                r"^(?:[a-zA-Z0-9_](?:[a-zA-Z0-9_-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,63}$",
            )
            .unwrap();

            if re.is_match(s) {
                Ok(())
            } else {
                Err(format!("Invalid hostname: {}", s))
            }
        }
        FieldType::Slug => {
            let s = value.as_str().ok_or("Not string for slug")?;
            let re = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
            if re.is_match(s) {
                Ok(())
            } else {
                Err(format!("Invalid slug: {}", s))
            }
        }
        FieldType::Hex => {
            let s = value.as_str().ok_or("Not string for hex")?;
            let re = Regex::new(r"^[0-9a-fA-F]+$").unwrap();
            if re.is_match(s) {
                Ok(())
            } else {
                Err(format!("Invalid hex: {}", s))
            }
        }
        FieldType::Base64 => {
            let s = value.as_str().ok_or("Not string for base64")?;
            let re = Regex::new(r"^[A-Za-z0-9+/]+={0,2}$").unwrap();
            if re.is_match(s) {
                Ok(())
            } else {
                Err(format!("Invalid base64: {}", s))
            }
        }
        FieldType::Password => {
            if value.as_str().is_some() {
                Ok(())
            } else {
                Err("Not string for password".into())
            }
        }
        FieldType::Token => {
            if value.as_str().is_some() {
                Ok(())
            } else {
                Err("Not string for token".into())
            }
        }
    }
}

pub fn validate_object(value: &mut Value, rules: &[FieldRule]) -> Result<(), String> {
    if let Value::Object(_) = value {
        for rule in rules {
            validate_field(value, rule)?;
        }
        Ok(())
    } else {
        Err("Value is not object".into())
    }
}

pub fn validate_rule(rule_str: &str, value_str: &str) -> bool {
    // 1. 词法分析
    let tokens = match tokenize(rule_str) {
        Ok(t) => t,
        Err(_) => {
            return false;
        }
    };

    // 2. 解析完整规则 (nameless = false)
    let mut parser = Parser::new(tokens);
    let rule_ast = match parser.parse_field(false) {
        Ok(r) => r,
        Err(_) => {
            return false;
        }
    };

    // 3. 将输入字符串转换为对应的 Value 枚举
    let val_enum = match convert_input_to_value(value_str, &rule_ast.field_type) {
        Ok(v) => v,
        Err(_) => {
            return false;
        }
    };

    // 4. 构造上下文对象供 Validator 查找对应字段
    let mut map = HashMap::new();
    map.insert(rule_ast.field.clone(), val_enum);
    let mut wrapped_value = Value::Object(map);

    // 5. 执行验证
    validate_field(&mut wrapped_value, &rule_ast).is_ok()
}

fn convert_input_to_value(input: &str, target_type: &FieldType) -> Result<Value, String> {
    match target_type {
        FieldType::Int => input
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|e| e.to_string()),
        FieldType::Float => input
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|e| e.to_string()),
        FieldType::Bool => {
            // 统一转为小写进行不区分大小写的匹配
            match input.to_lowercase().as_str() {
                "true" | "1" => Ok(Value::Bool(true)),
                "false" | "0" => Ok(Value::Bool(false)),
                _ => Err("Invalid boolean value".into()),
            }
        }
        _ => Ok(Value::String(input.to_string())),
    }
}
