#[cfg(test)]
mod tests {
    use zz_validator::ast::{Constraint, Constraints, FieldRule, FieldType, Value};

    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_field_type_variants() {
        // 覆盖 FieldType 所有变体及其 Debug/PartialEq 派生
        let types = vec![
            FieldType::String, FieldType::Int, FieldType::Float, FieldType::Bool,
            FieldType::Object, FieldType::Array, FieldType::Email, FieldType::Uri,
            FieldType::Uuid, FieldType::Ip, FieldType::Mac, FieldType::Date,
            FieldType::DateTime, FieldType::Time, FieldType::Timestamp,
            FieldType::Color, FieldType::Hostname, FieldType::Slug,
            FieldType::Hex, FieldType::Base64, FieldType::Password, FieldType::Token,
        ];

        for t in &types {
            let cloned = t.clone();
            assert_eq!(t, &cloned);
            assert!(format!("{:?}", t).contains(format!("{:?}", cloned).as_str()));
        }
    }

    #[test]
    fn test_value_conversions() {
        // 1. String
        let mut v_str = Value::String("hello".to_string());
        assert_eq!(v_str.as_str(), Some("hello"));
        assert_eq!(v_str.as_int(), None); // 测试错误分支
        assert!(format!("{:?}", v_str).contains("String"));

        // 2. Int
        let v_int = Value::Int(42);
        assert_eq!(v_int.as_int(), Some(42));
        assert_eq!(v_int.as_str(), None);

        // 3. Float
        let v_float = Value::Float(3.14);
        assert_eq!(v_float.as_float(), Some(3.14));
        assert_eq!(v_float.as_bool(), None);

        // 4. Bool
        let v_bool = Value::Bool(true);
        assert_eq!(v_bool.as_bool(), Some(true));
        assert_eq!(v_bool.as_float(), None);

        // 5. Object & Mut
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Int(1));
        let mut v_obj = Value::Object(map);
        assert!(v_obj.as_object().is_some());
        assert_eq!(v_obj.as_object().unwrap().get("key"), Some(&Value::Int(1)));
        assert!(v_obj.as_object_mut().is_some());
        assert!(v_obj.as_array().is_none());

        // 6. Array & Mut
        let mut v_arr = Value::Array(vec![Value::Bool(false)]);
        assert!(v_arr.as_array().is_some());
        assert_eq!(v_arr.as_array().unwrap()[0], Value::Bool(false));
        assert!(v_arr.as_array_mut().is_some());
        assert!(v_arr.as_object_mut().is_none());
    }

    #[test]
    fn test_constraints_and_clone() {
        let c = Constraint::Range {
            min: Value::Int(0),
            max: Value::Int(10),
            min_inclusive: true,
            max_inclusive: false,
        };
        let regex = Constraint::Regex(".*".to_string());
        
        let constraints = Constraints {
            items: vec![c.clone(), regex.clone()],
        };

        // 触发 Clone 和 Debug
        let cloned_constraints = constraints.clone();
        let debug_str = format!("{:?}", cloned_constraints);
        assert!(debug_str.contains("Range"));
        assert!(debug_str.contains("Regex"));
    }

    #[test]
    fn test_field_rule_structure() {
        // 测试复杂的递归结构以覆盖所有字段
        let rule = FieldRule {
            field: "username".to_string(),
            field_type: FieldType::String,
            required: true,
            default: Some(Value::String("guest".to_string())),
            enum_values: Some(vec![Value::String("admin".to_string())]),
            union_types: Some(vec![FieldType::String, FieldType::Int]),
            constraints: Some(Constraints {
                items: vec![Constraint::Regex("^[a-z]+$".to_string())],
            }),
            rule: Some(Box::new(FieldRule {
                field: "sub".to_string(),
                field_type: FieldType::Int,
                required: false,
                default: None,
                enum_values: None,
                union_types: None,
                constraints: None,
                rule: None,
                children: None,
                is_array: false,
            })),
            children: Some(vec![]),
            is_array: false,
        };

        let cloned_rule = rule.clone();
        assert_eq!(cloned_rule.field, "username");
        assert!(format!("{:?}", rule).contains("username"));
    }
}