#[cfg(test)]
mod coverage_boost_tests {
    use std::collections::HashMap;
    use zz_validator::{
        ast::{FieldType, Value, FieldRule, Constraints, Constraint},
        token::{tokenize, Token},
        parser::Parser,
        validator::{validate_field, validate_type, validate_object, validate_rule, ValidationError},
    };

    // -------------------------------------------------------------------------
    // 1. Tokenizer 边界覆盖
    // -------------------------------------------------------------------------
    #[test]
    fn test_tokenizer_edge_cases() {
        // 非法数字
        assert!(tokenize("1.2.3").is_err());
        assert!(tokenize("1e2e3").is_err());
        
        // 转义字符覆盖
        let tokens = tokenize(r#""line1\nline2\rline3\tline4\"quote\\slash\other""#).unwrap();
        if let Token::Ident(s) = &tokens[0] {
            assert!(s.contains('\n'));
            assert!(s.contains('\r'));
            assert!(s.contains('\t'));
            assert!(s.contains('"'));
            assert!(s.contains('\\'));
            assert!(s.contains('o')); // other character
        }

        // 意外字符
        assert!(tokenize("field: @int").is_err());
    }

    // -------------------------------------------------------------------------
    // 2. Parser 逻辑与错误路径覆盖
    // -------------------------------------------------------------------------
    #[test]
    fn test_parser_coverage() {
        // parse_rules 必须以 ( 开头
        assert!(Parser::parse_rules("u:int").is_err());

        // Unknown type
        assert!(Parser::parse_rules("(field:unknown)").is_err());

        // Expected type, got something else
        assert!(Parser::parse_rules("(field: :)").is_err());

        // Invalid range numbers
        assert!(Parser::parse_rules("(field:int[abc, 10])").is_err());
        assert!(Parser::parse_rules("(field:int[1, def])").is_err());

        // Unexpected tokens in enum
        assert!(Parser::parse_rules("(field:string enum()").is_err());
        assert!(Parser::parse_rules("(field:string enum(a, ))").is_err());

        // Parser expect fail (through public API)
        assert!(Parser::parse_rules("(field int)").is_err()); // Missing colon

        // LT but not array
        assert!(Parser::parse_rules("(field:int<sub:string>)").is_err());

        // LT but nothing after
        assert!(Parser::parse_rules("(field:array<)").is_err());

        // LParen but nothing after
        assert!(Parser::parse_rules("(field:object()").is_err());

        // Union types 覆盖: 包含多个类型
        let rules = Parser::parse_rules("(u:int|bool|string)").unwrap();
        assert_eq!(rules[0].union_types.as_ref().unwrap().len(), 3);

        // 复杂嵌套 + Union 覆盖 (注意: array 不支持直接加 range，子项支持)
        let rules = Parser::parse_rules("(a:array<int|string[1, 5]>)").unwrap();
        assert!(rules[0].is_array);
        assert!(rules[0].rule.is_some());
    }

    // -------------------------------------------------------------------------
    // 3. AST (Value / FieldType / Constraints) 覆盖
    // -------------------------------------------------------------------------
    #[test]
    fn test_ast_conversions_and_clone() {
        let mut v = Value::Object(HashMap::new());
        assert!(v.as_object_mut().is_some());
        assert!(v.as_object().is_some());
        assert!(v.as_str().is_none());
        assert!(v.as_int().is_none());
        assert!(v.as_float().is_none());
        assert!(v.as_bool().is_none());
        assert!(v.as_array().is_none());
        assert!(v.as_array_mut().is_none());

        let mut arr = Value::Array(vec![]);
        assert!(arr.as_array_mut().is_some());
        assert!(arr.as_array().is_some());

        // Constraints Clone 覆盖
        let c = Constraints { items: vec![Constraint::Regex(".*".into())] };
        let _c2 = c.clone();
        let _c3 = Constraint::Range { min: Value::Int(1), max: Value::Int(10), min_inclusive: true, max_inclusive: true }.clone();
        
        // Debug 覆盖
        let _ = format!("{:?} {:?} {:?}", FieldType::String, v, c);
    }

    // -------------------------------------------------------------------------
    // 4. Validator 逻辑与错误展示覆盖
    // -------------------------------------------------------------------------
    #[test]
    fn test_validator_deep_coverage() {
        // Hostname 边界校验
        let long_hostname = "a".repeat(254);
        assert!(validate_type(&Value::String(long_hostname), &FieldType::Hostname).is_err());
        assert!(validate_type(&Value::String("".into()), &FieldType::Hostname).is_err());
        assert!(validate_type(&Value::Int(123), &FieldType::Hostname).is_err());

        // Password / Token 类型校验
        assert!(validate_type(&Value::String("pwd".into()), &FieldType::Password).is_ok());
        assert!(validate_type(&Value::Int(123), &FieldType::Password).is_err());
        assert!(validate_type(&Value::String("tok".into()), &FieldType::Token).is_ok());

        // Timestamp 校验
        assert!(validate_type(&Value::Int(123), &FieldType::Timestamp).is_ok());
        assert!(validate_type(&Value::String("".into()), &FieldType::Timestamp).is_err());

        // Default 注入逻辑覆盖
        let rule = FieldRule {
            field: "opt".into(),
            field_type: FieldType::Int,
            required: false,
            default: Some(Value::Int(42)),
            enum_values: None,
            union_types: None,
            constraints: None,
            rule: None,
            children: None,
            is_array: false,
        };
        let mut obj = Value::Object(HashMap::new());
        validate_field(&mut obj, &rule).unwrap();
        assert_eq!(obj.as_object().unwrap().get("opt").unwrap().as_int().unwrap(), 42);

        // 可选字段空字符串跳过逻辑覆盖
        let rule_opt_str = FieldRule {
            field: "s".into(),
            field_type: FieldType::String,
            required: false,
            default: None,
            enum_values: None,
            union_types: None,
            constraints: None,
            rule: None,
            children: None,
            is_array: false,
        };
        let mut val_empty = Value::String("".into());
        assert!(validate_field(&mut val_empty, &rule_opt_str).is_ok());

        // Regex 编译错误路径
        let rule_re = FieldRule {
            field: "re".into(),
            field_type: FieldType::String,
            required: true,
            default: None,
            enum_values: None,
            union_types: None,
            constraints: Some(Constraints { items: vec![Constraint::Regex("[".into())] }),
            rule: None,
            children: None,
            is_array: false,
        };
        let mut data = Value::Object(HashMap::from([("re".into(), Value::String("a".into()))]));
        assert!(validate_field(&mut data, &rule_re).is_err());

        // validate_object 非对象覆盖
        assert!(validate_object(&mut Value::Int(1), &[]).is_err());

        // Union Type 报错覆盖
        let rule_union = FieldRule {
            field: "u".into(),
            field_type: FieldType::Int,
            required: true,
            default: None,
            enum_values: None,
            union_types: Some(vec![FieldType::Int, FieldType::Bool]),
            constraints: None,
            rule: None,
            children: None,
            is_array: false,
        };
        let mut data_union = Value::Object(HashMap::from([("u".into(), Value::String("s".into()))]));
        assert!(validate_field(&mut data_union, &rule_union).is_err());

        // 递归 array 覆盖
        let rule_arr = FieldRule {
            field: "tags".into(),
            field_type: FieldType::Array,
            required: true,
            default: None,
            enum_values: None,
            union_types: None,
            constraints: None,
            rule: Some(Box::new(FieldRule {
                field: "".into(), 
                field_type: FieldType::Int,
                required: true,
                default: None,
                enum_values: None,
                union_types: None,
                constraints: None,
                rule: None,
                children: None,
                is_array: false,
            })),
            children: None,
            is_array: true,
        };
        let mut data_arr = Value::Object(HashMap::from([("tags".into(), Value::Array(vec![Value::Int(1), Value::String("2".into())]))]));
        assert!(validate_field(&mut data_arr, &rule_arr).is_err());

        // validate_rule 覆盖
        assert!(validate_rule("a:int", "1"));
        assert!(!validate_rule("a:int", "abc"));
        assert!(!validate_rule("a:int", ""));
        assert!(validate_rule("a:bool", "true"));
        assert!(validate_rule("a:bool", "1"));
        assert!(validate_rule("a:bool", "false"));
        assert!(validate_rule("a:bool", "0"));
        assert!(!validate_rule("a:bool", "maybe"));
        assert!(!validate_rule("a:float", "notfloat"));
        assert!(!validate_rule("!!!", "1"));
        assert!(!validate_rule("a:int enum(1,2)", "3"));
    }

    #[test]
    fn test_validation_error_display_coverage() {
        let errs = vec![
            ValidationError::MissingField("f".into()),
            ValidationError::TypeMismatch { field: "f".into(), value: "v".into(), expected: "Int".into(), actual: "err".into() },
            ValidationError::UnionTypeMismatch { field: "f".into(), value: "v".into(), types: vec![FieldType::Int] },
            ValidationError::EnumMismatch { field: "f".into(), value: "v".into(), expected: vec![Value::Int(1)] },
            ValidationError::RangeError { field: "f".into(), value: "v".into(), min: "1".into(), max: "10".into() },
            ValidationError::RegexMismatch { field: "f".into(), pattern: "p".into() },
            ValidationError::InvalidRegex("err".into()),
            ValidationError::NotAnObject("f".into()),
            ValidationError::Custom("err".into()),
        ];

        for e in errs {
            let s = e.to_string();
            assert!(!s.is_empty());
            let _ = format!("{:?}", e);
        }
    }
}
