
#[cfg(test)]
mod tests {
    use zz_validator::{ast::Value, parser::Parser, validator::validate_object};


    #[test]
    fn test_validator_full_dsl() {
        let dsl =
            r#"
        (
            username:string[3,20] regex("^[a-zA-Z0-9_]+$"),
            age:int[0,150]=30,
            score:float(0,100),
            active:bool=true,
            nickname?:string[0,20],
            role:string enum("admin","user","guest")=user,
            id:int|float,
            profile:object(
                first_name:string[1,50],
                last_name:string[1,50],
                contact:object(
                    email:string regex("^[^@\\s]+@[^@\\s]+\\.[^@\\s]+$"),
                    phone?:string[0,20]
                )
            ),
            tags:array<string[1,10]>,
            distance:float[1.0e0,2.0e0]=1.5e0
        )
        "#;

        let rules = Parser::parse_rules(dsl).expect("Failed to parse DSL");

        let mut obj = Value::Object(Default::default());

        // 填充合法值
        obj.as_object_mut()
            .unwrap()
            .insert("username".to_string(), Value::String("user_123".to_string()));
        obj.as_object_mut().unwrap().insert("age".to_string(), Value::Int(25));
        obj.as_object_mut().unwrap().insert("score".to_string(), Value::Float(85.5));
        obj.as_object_mut().unwrap().insert("active".to_string(), Value::Bool(true));
        obj.as_object_mut().unwrap().insert("role".to_string(), Value::String("admin".to_string()));
        obj.as_object_mut().unwrap().insert("id".to_string(), Value::Int(101));

        // profile 对象
        let mut profile = Value::Object(Default::default());
        profile
            .as_object_mut()
            .unwrap()
            .insert("first_name".to_string(), Value::String("John".to_string()));
        profile
            .as_object_mut()
            .unwrap()
            .insert("last_name".to_string(), Value::String("Doe".to_string()));
        let mut contact = Value::Object(Default::default());
        contact
            .as_object_mut()
            .unwrap()
            .insert("email".to_string(), Value::String("john@example.com".to_string()));
        profile.as_object_mut().unwrap().insert("contact".to_string(), contact);
        obj.as_object_mut().unwrap().insert("profile".to_string(), profile);

        // tags 数组
        obj.as_object_mut()
            .unwrap()
            .insert(
                "tags".to_string(),
                Value::Array(
                    vec![Value::String("tag1".to_string()), Value::String("tag2".to_string())]
                )
            );

        // 调用 validator
        let res = validate_object(&mut obj, &rules);
        assert!(res.is_ok(), "Validation failed: {:?}", res.err());

        // 默认值填充
        assert_eq!(obj.as_object().unwrap().get("distance"), Some(&Value::Float(1.5)));

        // 错误测试 - 类型不匹配
        let mut bad_obj = obj.clone();
        bad_obj
            .as_object_mut()
            .unwrap()
            .insert("age".to_string(), Value::String("not_a_number".to_string()));
        let err = validate_object(&mut bad_obj, &rules).unwrap_err();
        println!("err = {:?}", err);
        assert!(err.contains("age value"), "Expected age type error, got {}", err);

        // 错误测试 - enum 不匹配
        let mut bad_enum = obj.clone();
        bad_enum
            .as_object_mut()
            .unwrap()
            .insert("role".to_string(), Value::String("superuser".to_string()));
        let err = validate_object(&mut bad_enum, &rules).unwrap_err();
        assert!(err.contains("role value"), "Expected role enum error, got {}", err);

        // 错误测试 - regex 不匹配
        let mut bad_regex = obj.clone();
        bad_regex
            .as_object_mut()
            .unwrap()
            .insert("username".to_string(), Value::String("!!invalid!!".to_string()));
        let err = validate_object(&mut bad_regex, &rules).unwrap_err();
        assert!(err.contains("username regex mismatch"), "Expected regex error, got {}", err);

        // 错误测试 - range 不匹配
        let mut bad_range = obj.clone();
        bad_range.as_object_mut().unwrap().insert("score".to_string(), Value::Float(150.0));
        let err = validate_object(&mut bad_range, &rules).unwrap_err();
        assert!(err.contains("score value"), "Expected range error, got {}", err);
    }

        #[test]
    fn test_special_types() {
        let dsl = r#"(email?:email, id:uuid, homepage:uri)"#;
        let rules = Parser::parse_rules(dsl).expect("Failed to parse DSL");

        let mut obj = Value::Object(Default::default());
        obj.as_object_mut().unwrap().insert("email".to_string(), Value::String("user@example.com".to_string()));
        obj.as_object_mut().unwrap().insert("id".to_string(), Value::String("550e8400-e29b-41d4-a716-446655440000".to_string()));
        obj.as_object_mut().unwrap().insert("homepage".to_string(), Value::String("https://example.com".to_string()));

        let res = validate_object(&mut obj, &rules);
        assert!(res.is_ok());
    }
}
