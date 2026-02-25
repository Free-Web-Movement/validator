#[cfg(test)]
mod string_range_error_tests {
    use std::collections::HashMap;

    use zz_validator::{ast::{Constraint, Constraints, FieldRule, Value}, parser::Parser, token::tokenize, validator::validate_field};


    fn parse_rule(rule_str: &str) -> FieldRule {
        let tokens = tokenize(rule_str).unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse_field(false).unwrap()
    }

    // --- 1. 字符串长度太短 (小于 min) ---
    #[test]
    fn test_err_string_length_min() {
        // 规则：长度必须在 [3, 10]
        let rule = parse_rule("username:string[3, 10]");
        let mut map = HashMap::new();
        map.insert("username".to_string(), Value::String("ab".into())); // 长度为 2
        let mut data = Value::Object(map);
        
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("length 2 out of range"));
    }

    // --- 2. 字符串长度太长 (大于 max) ---
    #[test]
    fn test_err_string_length_max() {
        // 规则：长度必须在 [1, 5]
        let rule = parse_rule("tag:string[1, 5]");
        let mut map = HashMap::new();
        map.insert("tag".to_string(), Value::String("abcdef".into())); // 长度为 6
        let mut data = Value::Object(map);
        
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("length 6 out of range"));
    }

    // --- 3. 开区间边界测试 (Exclusive boundary) ---
    #[test]
    fn test_err_string_length_exclusive() {
        // 规则：长度在 (3, 5] 之间，即长度必须 > 3
        let rule = parse_rule("code:string(3, 5]");
        let mut map = HashMap::new();
        map.insert("code".to_string(), Value::String("abc".into())); // 长度刚好为 3，本应报错
        let mut data = Value::Object(map);
        
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("length 3 out of range"));
    }

    // --- 4. 非法边界类型：String 范围给了 Float 边界 ---
    // 覆盖代码中的: _ => { return Err(format!("Invalid min value type in range for {}", rule.field)); }
    #[test]
    fn test_err_string_range_invalid_boundary_type() {
        let mut rule = parse_rule("name:string");
        // 构造非法规则：给 string 长度约束，但边界给了 Float 类型
        rule.constraints = Some(Constraints {
            items: vec![Constraint::Range {
                min: Value::Float(1.5), // 🚩 长度边界不能是浮点数
                max: Value::Int(10),
                min_inclusive: true,
                max_inclusive: true,
            }]
        });

        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::String("test".into()));
        let mut data = Value::Object(map);
        
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Invalid min value type in range"));
    }

    // --- 5. 边界字符串解析失败 ---
    // 覆盖代码中的: s.parse::<usize>().map_err(|_| format!("Failed to parse '{}' as usize", s))?
    #[test]
    fn test_err_string_range_parse_fail() {
        let mut rule = parse_rule("note:string");
        // 构造规则：min 边界是一个无法转为数字的字符串
        rule.constraints = Some(Constraints {
            items: vec![Constraint::Range {
                min: Value::String("five".into()), // 🚩 无法解析为 usize
                max: Value::Int(10),
                min_inclusive: true,
                max_inclusive: true,
            }]
        });

        let mut map = HashMap::new();
        map.insert("note".to_string(), Value::String("hello".into()));
        let mut data = Value::Object(map);
        
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Failed to parse 'five' as usize"));
    }

    // --- 1. 字符串长度超过闭区间最大值 [min, max] ---
    #[test]
    fn test_err_string_length_max_inclusive() {
        // 规则：长度必须在 [1, 3] 之间
        let rule = parse_rule("code:string[1, 3]");
        let mut map = HashMap::new();
        // 错误案例：长度为 4，超过 3
        map.insert("code".to_string(), Value::String("abcd".into()));
        let mut data = Value::Object(map);
        
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("length 4 out of range"));
    }

    // --- 2. 字符串长度触发开区间最大值边界 [min, max) ---
    #[test]
    fn test_err_string_length_max_exclusive() {
        // 规则：长度必须在 [1, 3) 之间，即长度必须 < 3（实际只能是 1 或 2）
        let rule = parse_rule("code:string[1, 3)");
        let mut map = HashMap::new();
        // 错误案例：长度为 3，但在开区间上限处是不允许的
        map.insert("code".to_string(), Value::String("abc".into()));
        let mut data = Value::Object(map);
        
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("length 3 out of range"));
    }

    // --- 3. 非法 Max 边界类型：String 长度上限给了 Float 边界 ---
    // 覆盖代码中：_ => { return Err(format!("Invalid max value type in range for {}", rule.field)); }
    #[test]
    fn test_err_string_max_invalid_boundary_type() {
        let mut rule = parse_rule("note:string");
        // 手动构造非法规则：给 string 长度约束，但 max 边界给了 Float
        rule.constraints = Some(Constraints {
            items: vec![Constraint::Range {
                min: Value::Int(1),
                max: Value::Float(5.5), // 🚩 字符串长度上限不能是浮点数
                min_inclusive: true,
                max_inclusive: true,
            }]
        });

        let mut map = HashMap::new();
        map.insert("note".to_string(), Value::String("hello!".into()));
        let mut data = Value::Object(map);
        
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Invalid max value type in range"));
    }

    // --- 4. Max 边界字符串解析失败 ---
    // 覆盖代码中：s.parse::<usize>().map_err(|_| format!("Failed to parse '{}' as usize", s))?
    #[test]
    fn test_err_string_max_parse_fail() {
        let mut rule = parse_rule("comment:string");
        // 构造规则：max 边界是一个无法转为数字的字符串
        rule.constraints = Some(Constraints {
            items: vec![Constraint::Range {
                min: Value::Int(0),
                max: Value::String("ten".into()), // 🚩 无法解析为 usize
                min_inclusive: true,
                max_inclusive: true,
            }]
        });

        let mut map = HashMap::new();
        map.insert("comment".to_string(), Value::String("This is a test".into()));
        let mut data = Value::Object(map);
        
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Failed to parse 'ten' as usize"));
    }

    #[test]
fn test_default_value_parsing_no_crash() {
    let dsl = "status:string = 200";
    let tokens = tokenize(dsl).unwrap();
    let mut parser = Parser::new(tokens);
    let rule = parser.parse_field(false).unwrap();
    
    // 验证数字默认值是否成功转为了 Value::String
    assert!(matches!(rule.default, Some(Value::String(ref s)) if s == "200"));
}
}