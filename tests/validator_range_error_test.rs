#[cfg(test)]
mod range_error_tests {
    use std::collections::HashMap;

    use zz_validator::{
        ast::{Constraint, Constraints, FieldRule, Value},
        parser::Parser,
        token::tokenize,
        validator::validate_field,
    };

    fn parse_rule(rule_str: &str) -> FieldRule {
        let tokens = tokenize(rule_str).unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse_field(false).unwrap()
    }

    // --- 1. 整数范围越界 ---
    #[test]
    fn test_err_range_int_out_of_bounds() {
        let rule = parse_rule("count:int[1, 10]");
        let mut map = HashMap::new();
        map.insert("count".to_string(), Value::Int(11)); // 越过最大值
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("out of range"));
    }

    #[test]
    fn test_err_int_min_rounding() {
        // 规则：int[1.2, 10] -> 实际有效最小值是 2
        let rule = parse_rule("count:int[1.2, 10]");
        let mut map = HashMap::new();

        // 错误案例：输入 1。虽然 1 > 1.2 是假的，
        // 但根据逻辑，min_v 变成了 2.0，1 < 2.0 触发错误。
        map.insert("count".to_string(), Value::Int(1));
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err(), "1 should be less than rounded min 2");
    }

    #[test]
    fn test_err_int_max_rounding() {
        // 规则：int[0, 5.8] -> 实际有效最大值是 5
        let rule = parse_rule("count:int[0, 5.8]");
        let mut map = HashMap::new();

        // 错误案例：输入 6。
        // 根据逻辑，max_v 变成了 5.0，6 > 5.0 触发错误。
        map.insert("count".to_string(), Value::Int(6));
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err(), "6 should be greater than rounded max 5");
    }

    #[test]
    fn test_int_boundary_ok() {
        // 验证边界值是否可以通过
        let rule = parse_rule("count:int[1.2, 5.8]");

        // 2 应该通过 (ceil(1.2) = 2)
        let mut data2 = Value::Object(HashMap::from([("count".into(), Value::Int(2))]));
        assert!(validate_field(&mut data2, &rule).is_ok());

        // 5 应该通过 (floor(5.8) = 5)
        let mut data5 = Value::Object(HashMap::from([("count".into(), Value::Int(5))]));
        assert!(validate_field(&mut data5, &rule).is_ok());
    }

    // --- 2. 字符串长度越界 ---
    #[test]
    fn test_err_range_string_length_out_of_bounds() {
        let rule = parse_rule("name:string[2, 4]");
        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::String("a".into())); // 太短
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("length 1 out of range"));
    }

    // --- 3. 非法边界类型：Int 范围却给了 String 边界 ---
    // 代码中：_ => return Err(format!("Invalid min value type in range for {}", rule.field));
    #[test]
    fn test_err_range_invalid_boundary_type() {
        let mut rule = parse_rule("count:int");
        rule.constraints = Some(Constraints {
            items: vec![Constraint::Range {
                min: Value::String("0".into()),
                max: Value::Int(10),
                min_inclusive: true,
                max_inclusive: true,
            }],
        });

        let mut map = HashMap::new();
        map.insert("count".to_string(), Value::Int(5));
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());

        let err_msg = res.unwrap_err();
        // 打印出来看看实际的报错是什么，方便调试
        // println!("Actual error: {}", err_msg);

        // 修正断言：使用源代码中肯定存在的关键字
        assert!(err_msg.contains("Invalid min value type"));
        // 如果你的代码里确实有字段名，也可以加上：
        assert!(err_msg.contains("count"));
    }

    // --- 4. 无法应用 Range 的类型 (例如对 Bool 使用 Range) ---
    // 代码中：_ => return Err(format!("{} cannot apply range constraint to {:?}", rule.field, val));
    #[test]
    fn test_err_range_unsupported_type() {
        // 强制给 bool 加上 range 约束
        let mut rule = parse_rule("active:bool");
        rule.constraints = Some(Constraints {
            items: vec![Constraint::Range {
                min: Value::Int(0),
                max: Value::Int(1),
                min_inclusive: true,
                max_inclusive: true,
            }],
        });

        let mut map = HashMap::new();
        map.insert("active".to_string(), Value::Bool(true));
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("cannot apply range constraint"));
    }

    // --- 5. 字符串长度解析失败 ---
    // 代码中：s.parse::<usize>().map_err(|_| format!("Failed to parse '{}' as usize", s))?
    #[test]
    fn test_err_range_string_boundary_parse_fail() {
        // 模拟 min/max 字符串不是数字的情况
        let mut rule = parse_rule("name:string");
        rule.constraints = Some(Constraints {
            items: vec![Constraint::Range {
                min: Value::String("abc".into()), // 🚩 无法解析为 usize
                max: Value::Int(10),
                min_inclusive: true,
                max_inclusive: true,
            }],
        });

        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::String("test".into()));
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Failed to parse 'abc' as usize"));
    }

    #[test]
    fn test_err_int_max_inclusive() {
        // 规则：count 必须在 [1, 10] 之间
        let rule = parse_rule("count:int[1, 10]");
        let mut map = HashMap::new();

        // 错误案例：超过最大值 10
        map.insert("count".to_string(), Value::Int(11));
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        // 校验错误信息是否包含字段名、非法值以及范围描述
        let err = res.unwrap_err();
        assert!(err.contains("count"));
        assert!(err.contains("11"));
        assert!(err.contains("out of range"));
    }

    #[test]
    fn test_err_int_max_exclusive() {
        // 规则：count 必须在 [1, 10) 之间 (即最大只能是 9)
        let rule = parse_rule("count:int[1, 10)");
        let mut map = HashMap::new();

        // 错误案例：等于 10，但在开区间下是不允许的
        map.insert("count".to_string(), Value::Int(10));
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("out of range"));
    }

    #[test]
    fn test_err_int_max_invalid_type() {
        // 场景：Int 的上限 max 给了一个非数字类型（如 String）
        // 触发代码中的: _ => { return Err(format!("Invalid max value type in range for {}", rule.field)); }
        let mut rule = parse_rule("count:int");
        rule.constraints = Some(Constraints {
            items: vec![Constraint::Range {
                min: Value::Int(0),
                max: Value::String("not_a_number".into()), // 🚩 非法上限类型
                min_inclusive: true,
                max_inclusive: true,
            }],
        });

        let mut map = HashMap::new();
        map.insert("count".to_string(), Value::Int(5));
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Invalid max value type"));
    }

    // --- 1. 类型不匹配 (Not float) ---
    #[test]
    fn test_err_float_type_mismatch() {
        let rule = parse_rule("price:float");
        let mut map = HashMap::new();
        // 传入的是布尔值，不是浮点数
        map.insert("price".to_string(), Value::Bool(true));
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Not float"));
    }

    // --- 2. 最小值越界 (Min boundary) ---
    #[test]
    fn test_err_float_min_out_of_bounds() {
        // 规则：(0.5, 10.0]
        let rule = parse_rule("weight:float(0.5, 10.0]");
        let mut map = HashMap::new();

        // 错误：等于 0.5，但由于是 '(' 开区间，要求必须 > 0.5
        map.insert("weight".to_string(), Value::Float(0.5));
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("out of range"));
    }

    // --- 3. 最大值越界 (Max boundary) ---
    #[test]
    fn test_err_float_max_out_of_bounds() {
        let rule = parse_rule("weight:float[0.0, 5.5]");
        let mut map = HashMap::new();

        // 错误：5.51 超过了 5.5
        map.insert("weight".to_string(), Value::Float(5.51));
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("out of range"));
    }

    // --- 4. 非法 Min 边界类型 (Invalid min value type) ---
    #[test]
    fn test_err_float_invalid_min_boundary() {
        let mut rule = parse_rule("val:float");
        // 手动构造：给 float 加上一个 String 类型的 min 边界
        rule.constraints = Some(Constraints {
            items: vec![Constraint::Range {
                min: Value::String("zero".into()),
                max: Value::Float(10.0),
                min_inclusive: true,
                max_inclusive: true,
            }],
        });

        let mut map = HashMap::new();
        map.insert("val".to_string(), Value::Float(5.0));
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Invalid min value type in range"));
    }

    // --- 5. 非法 Max 边界类型 (Invalid max value type) ---
    #[test]
    fn test_err_float_invalid_max_boundary() {
        let mut rule = parse_rule("val:float");
        // 手动构造：给 float 加上一个 Bool 类型的 max 边界
        rule.constraints = Some(Constraints {
            items: vec![Constraint::Range {
                min: Value::Float(0.0),
                max: Value::Bool(false),
                min_inclusive: true,
                max_inclusive: true,
            }],
        });

        let mut map = HashMap::new();
        map.insert("val".to_string(), Value::Float(5.0));
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Invalid max value type in range"));
    }
}
