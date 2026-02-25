#[cfg(test)]
mod validate_type_error_tests {
    use std::collections::HashMap;

    use zz_validator::{ast::{FieldType, Value}, validator::validate_type};
    
    // 辅助工具：快速构造 Value 进行测试
    fn check_type_err(val: Value, t: FieldType) -> String {
        validate_type(&val, &t).unwrap_err()
    }

    // --- 1. 基础物理类型物理错误 ---
    #[test]
    fn test_physical_type_errors() {
        // String 错误
        assert_eq!(check_type_err(Value::Int(1), FieldType::String), "Not string");
        // Int 错误
        assert_eq!(check_type_err(Value::String("1".into()), FieldType::Int), "Not int");
        // Float 错误
        assert_eq!(check_type_err(Value::Int(1), FieldType::Float), "Not float");
        // Bool 错误
        assert_eq!(check_type_err(Value::Int(1), FieldType::Bool), "Not bool");
        // Object 错误
        assert_eq!(check_type_err(Value::Array(vec![]), FieldType::Object), "Not object");
        // Array 错误
        assert_eq!(check_type_err(Value::Object(HashMap::new()), FieldType::Array), "Not array");
    }

    // --- 2. 语义类型：物理类型检查 (as_str/as_int 拦截) ---
    #[test]
    fn test_semantic_physical_intercepts() {
        // Email 必须是字符串
        assert_eq!(check_type_err(Value::Int(123), FieldType::Email), "Not string for email");
        // Timestamp 必须是整数
        assert_eq!(check_type_err(Value::String("2024".into()), FieldType::Timestamp), "Not number for timestamp");
        // Password/Token 必须是字符串
        assert_eq!(check_type_err(Value::Bool(true), FieldType::Password), "Not string for password");
    }

    // --- 3. 语义类型：格式校验 (Regex/Parser 拦截) ---
    #[test]
    fn test_semantic_format_errors() {
        // URI 错误 (使用 url crate 解析)
        assert!(check_type_err(Value::String("not-a-url".into()), FieldType::Uri).contains("is not a valid URI"));

        // MAC 地址错误
        assert!(check_type_err(Value::String("00:11:22:33:44:GG".into()), FieldType::Mac).contains("Invalid mac"));

        // Date 格式错误 (必须 YYYY-MM-DD)
        assert!(check_type_err(Value::String("2024/01/01".into()), FieldType::Date).contains("Invalid date"));

        // DateTime 格式错误
        assert!(check_type_err(Value::String("2024-01-01 12:00:00".into()), FieldType::DateTime).contains("Invalid datetime"));

        // Color 格式错误
        assert!(check_type_err(Value::String("rgb(0,0,0)".into()), FieldType::Color).contains("Invalid color"));

        // Hostname 格式错误
        assert!(check_type_err(Value::String("-bad-host.com".into()), FieldType::Hostname).contains("Invalid hostname"));

        // Slug 格式错误 (不能有大写或空格)
        assert!(check_type_err(Value::String("My Slug".into()), FieldType::Slug).contains("Invalid slug"));

        // Hex 格式错误
        assert!(check_type_err(Value::String("0xG1".into()), FieldType::Hex).contains("Invalid hex"));

        // Base64 格式错误 (包含非法字符)
        assert!(check_type_err(Value::String("Base64!".into()), FieldType::Base64).contains("Invalid base64"));
    }
}