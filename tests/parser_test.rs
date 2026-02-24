
#[cfg(test)]
mod parser_tests {
    use zz_validator::{ast::{Constraint, Constraints, FieldType, Value}, parser::Parser};
  
    #[test]
    fn test_full_dsl_parse() {
        let dsl =
            r#"
        (
            username:string[3,20] regex("^[a-zA-Z0-9_]+$"),  
            age:int[0,150]=30,      
            age:int=30,    
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
            scores:array<int[0,100]>,
            distance:float[1.47e11,1.52e11]=1.496e11,
            positive_scientific:float[+1.0e3,+2.0E3]=+1.5e3, 
            negative_scientific:float[-1.0e3,-2.0E3]=-1.5e3, 
            mixed_sign_scientific:float[-1.0e3,+2.0e3]=3.0e0,
            escaped_field:string regex("line1\nline2\rtab\tquote\"backslash\\"),
            _start_with_underscore:string[1,10]=5
        )
        "#;

        let rules = Parser::parse_rules(dsl).expect("Failed to parse DSL");

        // 检查总字段数量
        assert_eq!(rules.len(), 17);

        // 检查部分关键字段
        assert_eq!(rules[0].field, "username");
        assert_eq!(rules[0].field_type, FieldType::String);
        assert!(rules[0].constraints.is_some());

        assert_eq!(rules[16].field, "_start_with_underscore");
        assert_eq!(rules[16].field_type, FieldType::String);
        assert_eq!(rules[16].default, Some(Value::String("5".into())));

        let escaped_field_constraint = match &rules[15].constraints {
            Some(Constraints { items }) =>
                match &items[0] {
                    Constraint::Regex(p) => p,
                    _ => panic!("Expected regex constraint"),
                }
            _ => panic!("Expected constraints"),
        };
        println!("{:?}", escaped_field_constraint);
        // 确认转义字符仍存在
        assert!(escaped_field_constraint.contains(r"line1"));
        assert!(escaped_field_constraint.contains("\n"));
        // assert!(escaped_field_constraint.contains(r"quote"));
    }
}
