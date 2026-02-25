#[cfg(test)]
mod negative_tests {
    use std::collections::HashMap;

    use zz_validator::{
        ast::{FieldRule, Value},
        parser::Parser,
        token::tokenize,
        validator::validate_field,
    };

    /// 辅助函数：快速解析规则字符串
    fn parse_rule(rule_str: &str) -> FieldRule {
        let tokens = tokenize(rule_str).unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse_field(false).unwrap()
    }

    // --- 1. 必填字段缺失 (Missing required field) ---
    #[test]
    fn test_err_missing_required() {
        let rule = parse_rule("name:string");
        let mut data = Value::Object(HashMap::new()); // 数据中没有 "name"
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Missing required field"));
    }

    // --- 2. 联合类型不匹配 (Union types mismatch) ---
    #[test]
    fn test_err_union_mismatch() {
        let rule = parse_rule("id:int|float");
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::String("abc".into())); // 既不是 int 也不是 float
        let mut data = Value::Object(map);
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("does not match union types"));
    }

    // --- 3. 基础类型不匹配 (Type mismatch) ---
    #[test]
    fn test_err_type_mismatch() {
        let rule = parse_rule("age:int");
        let mut map = HashMap::new();
        // 模拟传入了字符串类型的 "25"，但规则要求是 int
        map.insert("age".to_string(), Value::String("25".into()));
        let mut data = Value::Object(map);

        let res = validate_field(&mut data, &rule);

        assert!(res.is_err());
        let err_msg = res.unwrap_err();

        // 匹配你代码中实际的 format! 格式
        // 格式为: "{field} value {val:?}: {err}"
        assert!(err_msg.contains("age"));
        assert!(err_msg.contains("Not int"));
        // 或者更精准的匹配：
        assert!(err_msg.contains("value String(\"25\"): Not int"));
    }
    // --- 4. 枚举值不存在 (Enum mismatch) ---
    #[test]
    fn test_err_enum_out_of_range() {
        let rule = parse_rule("color:string enum(red, blue)");
        let mut map = HashMap::new();
        map.insert("color".to_string(), Value::String("green".into()));
        let mut data = Value::Object(map);
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("not in enum"));
    }

    // --- 5. 数值范围越界 (Range out of range) ---
    #[test]
    fn test_err_int_range() {
        let rule = parse_rule("score:int[0, 100]");
        let mut map = HashMap::new();
        map.insert("score".to_string(), Value::Int(101));
        let mut data = Value::Object(map);
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("out of range"));
    }

    // --- 6. 字符串长度越界 (String length range) ---
    #[test]
    fn test_err_string_length() {
        let rule = parse_rule("pwd:string[6, 12]");
        let mut map = HashMap::new();
        map.insert("pwd".to_string(), Value::String("123".into())); // 太短
        let mut data = Value::Object(map);
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("length 3 out of range"));
    }

    // --- 7. 正则表达式不匹配 (Regex mismatch) ---
    #[test]
    fn test_err_regex_mismatch() {
        let rule = parse_rule(r#"code:string regex("^\d{3}$")"#);
        let mut map = HashMap::new();
        map.insert("code".to_string(), Value::String("12a".into()));
        let mut data = Value::Object(map);
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("regex mismatch"));
    }

    // --- 8. 递归验证错误：数组元素不符合子规则 ---
    #[test]
    fn test_err_array_sub_rule() {
        // 规则：tags 是一个字符串数组，每个字符串长度必须 [2, 5]
        let rule = parse_rule("tags:array<string[2, 5]>");
        let mut map = HashMap::new();
        map.insert(
            "tags".to_string(),
            Value::Array(vec![
                Value::String("ok".into()),
                Value::String("toolong".into()), // 这个会触发错误
            ]),
        );
        let mut data = Value::Object(map);
        let res = validate_field(&mut data, &rule);
        assert!(res.is_err());
    }

    // --- 9. 语义类型验证失败 (validate_type Err) ---
    #[test]
    fn test_err_semantic_types() {
        // 测试 Email 格式
        let rule_email = parse_rule("email:email");
        let mut map = HashMap::new();
        map.insert("email".to_string(), Value::String("invalid-email".into()));
        let mut data = Value::Object(map);
        assert!(validate_field(&mut data, &rule_email).is_err());

        // 测试 IP 格式
        let rule_ip = parse_rule("ip:ip");
        let mut map_ip = HashMap::new();
        map_ip.insert("ip".to_string(), Value::String("256.256.256.256".into()));
        let mut data_ip = Value::Object(map_ip);
        assert!(validate_field(&mut data_ip, &rule_ip).is_err());
    }

    // --- 10. 非对象却有子规则错误 ---
    #[test]
    fn test_err_not_object_but_has_children() {
        // 如果逻辑上定义了 children 但输入不是 Object
        let mut rule = parse_rule("user:int");
        rule.children = Some(vec![parse_rule("name:string")]);

        let mut val = Value::Int(10);
        let res = validate_field(&mut val, &rule);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("is not object but has children"));
    }
}
