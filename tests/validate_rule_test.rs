#[cfg(test)]
mod tests {
    use zz_validator::validator::validate_rule;
    // --- 基础解析测试 ---
    #[test]
    fn test_int_full_syntax_basic() {
        // 必填：name:type
        assert!(validate_rule("age:int", "25"));
        // 可选：name?:type
        assert!(validate_rule("score?:int", "90"));
    }

    // --- 范围约束测试 ---
    #[test]
    fn test_int_full_syntax_range() {
        // 闭区间 [min, max]
        let rule = "count:int[1, 10]";
        assert!(validate_rule(rule, "1"));
        assert!(validate_rule(rule, "10"));
        assert!(validate_rule(rule, "5"));
        assert!(!validate_rule(rule, "0"));
        assert!(!validate_rule(rule, "11"));

        // 开区间 (min, max)
        let rule_open = "price:int(10, 20)";
        assert!(!validate_rule(rule_open, "10"));
        assert!(!validate_rule(rule_open, "20"));
        assert!(validate_rule(rule_open, "15"));
    }

    // --- 默认值测试 ---
    #[test]
    fn test_int_full_syntax_default() {
        // name:type = default
        // 验证给定值依然必须符合类型要求
        let rule = "port:int = 8080";
        assert!(validate_rule(rule, "443"));
        assert!(!validate_rule(rule, "not_a_number"));
    }

    // --- 综合复杂测试 ---
    #[test]
    fn test_int_full_syntax_combined() {
        // <name>?:<type>[<constraints>] = <default>
        let rule = "level?:int[1, 99] = 1";

        // 合法值
        assert!(validate_rule(rule, "1"));
        assert!(validate_rule(rule, "50"));
        assert!(validate_rule(rule, "99"));

        // 非法值（不满足 range）
        assert!(!validate_rule(rule, "0"));
        assert!(!validate_rule(rule, "100"));

        // 类型错误
        assert!(!validate_rule(rule, "max"));
    }

    // --- 枚举测试 ---
    #[test]
    fn test_int_full_syntax_enum() {
        let rule = "code:int enum(200, 404, 500)";
        assert!(validate_rule(rule, "200"));
        assert!(validate_rule(rule, "500"));
        assert!(!validate_rule(rule, "201"));
    }

    #[test]
    fn test_int_range_boundary_mixtures() {
        // --- 情况 A: 左开右闭 (0, 100] ---
        // 规则含义：x > 0 且 x <= 100
        let rule_a = "val:int(0, 100]";
        assert!(!validate_rule(rule_a, "0"), "Left exclusive failed: 0 should be invalid");
        assert!(validate_rule(rule_a, "1"), "Left exclusive failed: 1 should be valid");
        assert!(validate_rule(rule_a, "100"), "Right inclusive failed: 100 should be valid");
        assert!(!validate_rule(rule_a, "101"), "Right inclusive failed: 101 should be invalid");

        // --- 情况 B: 左闭右开 [1, 100) ---
        // 规则含义：x >= 1 且 x < 100
        let rule_b = "val:int[1, 100)";
        assert!(validate_rule(rule_b, "1"), "Left inclusive failed: 1 should be valid");
        assert!(!validate_rule(rule_b, "0"), "Left inclusive failed: 0 should be invalid");
        assert!(validate_rule(rule_b, "99"), "Right exclusive failed: 99 should be valid");
        assert!(!validate_rule(rule_b, "100"), "Right exclusive failed: 100 should be invalid");
    }

    #[test]
    fn test_int_negative_ranges() {
        // --- 负数区间验证 [-10, -1) ---
        let rule_neg = "val:int[-10, -1)";
        assert!(validate_rule(rule_neg, "-10"));
        assert!(validate_rule(rule_neg, "-2"));
        assert!(!validate_rule(rule_neg, "-1"));
        assert!(!validate_rule(rule_neg, "0"));
    }

    #[test]
    fn test_int_single_point_range() {
        // --- 特殊情况：[5, 5] 应该只允许 5 ---
        let rule_point = "val:int[5, 5]";
        assert!(validate_rule(rule_point, "5"));
        assert!(!validate_rule(rule_point, "4"));
        assert!(!validate_rule(rule_point, "6"));

        // --- 特殊情况：(5, 5) 应该永远为 false ---
        let rule_empty = "val:int(5, 5)";
        assert!(!validate_rule(rule_empty, "5"));
    }

    #[test]
    fn test_mixed_brackets_parsing() {
        // 测试左开右闭
        let rule = "count:int(0, 10]";
        assert!(!validate_rule(rule, "0")); // 排除 0
        assert!(validate_rule(rule, "10")); // 包含 10

        // 测试左闭右开
        let rule2 = "count:int[0, 10)";
        assert!(validate_rule(rule2, "0")); // 包含 0
        assert!(!validate_rule(rule2, "10")); // 排除 10
    }

    // 浮点数测试

    #[test]
    fn test_float_basic() {
        // 基础浮点数校验
        assert!(validate_rule("price:float", "3.14"));
        assert!(validate_rule("temp:float", "-273.15"));
        assert!(validate_rule("count:float", "100")); // 整数应能自动转为 float 验证
        assert!(!validate_rule("price:float", "abc"));
    }

    #[test]
    fn test_float_scientific_notation() {
        // 科学计数法支持
        let rule = "val:float";
        assert!(validate_rule(rule, "1e10"));
        assert!(validate_rule(rule, "2.5E-3")); // 0.0025
        assert!(validate_rule(rule, "+1.23e+5"));
    }

    #[test]
    fn test_float_range_mixed() {
        // --- 混合区间：(0.0, 1.0] ---
        let rule_a = "prob:float(0.0, 1.0]";
        assert!(!validate_rule(rule_a, "0.0"), "Left exclusive failed");
        assert!(validate_rule(rule_a, "0.00001"));
        assert!(validate_rule(rule_a, "1.0"), "Right inclusive failed");
        assert!(!validate_rule(rule_a, "1.0001"));

        // --- 混合区间：[-5.5, 5.5) ---
        let rule_b = "range:float[-5.5, 5.5)";
        assert!(validate_rule(rule_b, "-5.5"));
        assert!(validate_rule(rule_b, "5.4999"));
        assert!(!validate_rule(rule_b, "5.5"));
    }

    #[test]
    fn test_float_enum() {
        // 浮点数枚举（虽然少见，但应支持）
        let rule = "ratio:float enum(0.5, 1.0, 2.0)";
        assert!(validate_rule(rule, "0.5"));
        assert!(validate_rule(rule, "1.0"));
        assert!(!validate_rule(rule, "0.7"));
    }

    #[test]
    fn test_float_complete_syntax() {
        // 完整格式：<name>?:<type>[<constraints>] = <default>
        let rule = "alpha?:float[0.1, 0.9] = 0.5";
        assert!(validate_rule(rule, "0.1"));
        assert!(validate_rule(rule, "0.9"));
        assert!(!validate_rule(rule, "0.05"));
    }

    #[test]
    fn test_bool_case_insensitivity() {
        let rule = "active:bool";

        // 各种大小写组合
        assert!(validate_rule(rule, "TRUE"));
        assert!(validate_rule(rule, "False"));
        assert!(validate_rule(rule, "tRuE"));

        // 混合数字和字符
        assert!(validate_rule(rule, "1"));
        assert!(validate_rule(rule, "0"));
    }

    #[test]
    fn test_bool_enum_with_case() {
        // 即使规则里定义的是小写 true，输入大写也应该匹配
        let rule = "flag:bool enum(true)";
        assert!(validate_rule(rule, "TRUE"));
        assert!(validate_rule(rule, "1"));
    }

    #[test]
    fn test_bool_negative_cases() {
        let rule = "active:bool";

        // 1. 常见的“类布尔”但不支持的单词
        assert!(!validate_rule(rule, "yes"), "Should not accept 'yes'");
        assert!(!validate_rule(rule, "no"), "Should not accept 'no'");
        assert!(!validate_rule(rule, "on"), "Should not accept 'on'");
        assert!(!validate_rule(rule, "off"), "Should not accept 'off'");

        // 2. 超出范围的数字
        assert!(!validate_rule(rule, "2"), "Should not accept '2'");
        assert!(!validate_rule(rule, "-1"), "Should not accept '-1'");
        assert!(!validate_rule(rule, "0.0"), "Should not accept float '0.0' for bool");

        // 3. 空值与乱码
        assert!(!validate_rule(rule, ""), "Empty string is not a bool");
        assert!(!validate_rule(rule, "  "), "Whitespace is not a bool");
        assert!(!validate_rule(rule, "null"), "Literal 'null' is not a bool");

        // 4. 拼写错误
        assert!(!validate_rule(rule, "ture"), "Typo 'ture' should fail");
        assert!(!validate_rule(rule, "flase"), "Typo 'flase' should fail");
    }

    #[test]
    fn test_bool_enum_negative_cases() {
        // 限制只能为 false 的规则
        let rule = "is_deleted:bool enum(false)";

        assert!(validate_rule(rule, "FALSE")); // 正常通过
        assert!(validate_rule(rule, "0")); // 正常通过

        assert!(!validate_rule(rule, "true"), "Should fail because enum only allows false");
        assert!(!validate_rule(rule, "1"), "Should fail because enum only allows false (via 1)");
    }

    // #[test]
    // fn test_string_basic_and_required() {
    //     // 必填字段：必须有值
    //     let rule_req = "username:string";
    //     assert!(validate_rule(rule_req, "Alice"));
    //     // 如果你的逻辑中规定必填字段传入 "" 视为缺失，则此处应为 false
    //     // 注意：这取决于 validate_field 内部对空值的处理
    //     assert!(!validate_rule(rule_req, ""), "Required field cannot be empty");
    // }

    #[test]
    fn test_string_optional_semantics() {
        // 可选字段：允许不包含数据
        let rule_opt = "nickname?:string[3, 10]";

        // 模拟不包含数据的情况
        assert!(validate_rule(rule_opt, ""), "Optional field can be empty");

        // 包含数据时必须符合约束
        assert!(validate_rule(rule_opt, "Alice"), "Valid length");
        assert!(!validate_rule(rule_opt, "Al"), "Invalid length (too short)");
    }

    #[test]
    fn test_string_length_boundaries() {
        // 长度区间 [2, 4]
        let rule = "code:string[2, 4]";
        assert!(validate_rule(rule, "ab")); // 长度 2
        assert!(validate_rule(rule, "abcd")); // 长度 4
        assert!(!validate_rule(rule, "a")); // 长度 1
        assert!(!validate_rule(rule, "abcde")); // 长度 5
    }

    // #[test]
    // fn test_string_regex() {
    //     // 正则：必须是 3 位数字
    //     let rule = r#"id:string regex("^\d{3}$")"#;
    //     assert!(validate_rule(rule, "123"));
    //     assert!(!validate_rule(rule, "12a"));
    //     assert!(!validate_rule(rule, "1234"));
    // }

    #[test]
    fn test_string_enum() {
        // 枚举：限定范围
        let rule = "mode:string enum(auto, manual, off)";
        assert!(validate_rule(rule, "auto"));
        assert!(validate_rule(rule, "off"));
        assert!(!validate_rule(rule, "on"));
    }

    #[test]
    fn test_string_default_value() {
        // 带默认值的可选字段
        let rule = "role?:string = \"guest\"";
        // 传入有效值
        assert!(validate_rule(rule, "admin"));
        // 传入空字符（模拟缺失，触发默认值）
        assert!(validate_rule(rule, ""), "Empty input should trigger default and pass");
    }
}
