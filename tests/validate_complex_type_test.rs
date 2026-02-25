#[cfg(test)]
mod validator_complex_type_tests {
    use zz_validator::{ast::FieldType, parser::Parser, token::tokenize};

    #[test]
    fn test_complex_nested_object_parsing() {
        let dsl = "profile:object(
        first_name:string[1,50],
        last_name:string[1,50],
        contact:object(
            email:email,
            phone?:string[0,20]
        )
    )";

        let tokens = tokenize(dsl).unwrap();
        let mut parser = Parser::new(tokens);
        let rule = parser.parse_field(false).unwrap();

        // 1. 验证顶层字段
        assert_eq!(rule.field, "profile");
        assert_eq!(rule.field_type, FieldType::Object);

        let children = rule.children.as_ref().expect("Should have children");
        assert_eq!(children.len(), 3);

        // 2. 验证 first_name
        assert_eq!(children[0].field, "first_name");
        assert_eq!(children[0].field_type, FieldType::String);

        // 3. 验证嵌套的 contact 对象
        let contact_rule = &children[2];
        assert_eq!(contact_rule.field, "contact");
        assert_eq!(contact_rule.field_type, FieldType::Object);

        let contact_children = contact_rule.children.as_ref().unwrap();
        assert_eq!(contact_children[0].field, "email");

        // 4. 验证可选字段 phone?
        assert_eq!(contact_children[1].field, "phone");
        assert_eq!(contact_children[1].required, false);
    }
}
