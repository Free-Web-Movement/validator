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
    assert!(!validate_rule(rule, "0"));  // 排除 0
    assert!(validate_rule(rule, "10"));  // 包含 10
    
    // 测试左闭右开
    let rule2 = "count:int[0, 10)";
    assert!(validate_rule(rule2, "0"));  // 包含 0
    assert!(!validate_rule(rule2, "10")); // 排除 10
}
}