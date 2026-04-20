use std::collections::HashMap;
use zz_validator::ast::Value;
use zz_validator::parser::Parser;
use zz_validator::validator::{validate, validate_type, ValidationError};
use zz_validator::ast::FieldType;

#[test]
fn test_validate_function_returns_option_value() {
    let rules = Parser::parse_rules("(name?:string)").unwrap();
    let data = Value::Object(HashMap::new());
    let result = validate(&data, &rules);
    assert!(result.is_some());
}

#[test]
fn test_validate_function_returns_none_on_failure() {
    let rules = Parser::parse_rules("(name:string[3,5])").unwrap();
    let data = Value::Object({
        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::String("toolongname".to_string()));
        map
    });
    let result = validate(&data, &rules);
    assert!(result.is_none());
}

#[test]
fn test_validate_type_phone_valid() {
    let val = Value::String("+1234567890".to_string());
    assert!(validate_type(&val, &FieldType::Phone).is_ok());
}

#[test]
fn test_validate_type_phone_invalid() {
    let val = Value::String("abc".to_string());
    assert!(validate_type(&val, &FieldType::Phone).is_err());
}

#[test]
fn test_validate_type_creditcard_valid() {
    let val = Value::String("4532015112830366".to_string());
    assert!(validate_type(&val, &FieldType::CreditCard).is_ok());
}

#[test]
fn test_validate_type_creditcard_invalid_length() {
    let val = Value::String("123".to_string());
    assert!(validate_type(&val, &FieldType::CreditCard).is_err());
}

#[test]
fn test_validate_type_creditcard_invalid_checksum() {
    let val = Value::String("1234567890123456".to_string());
    assert!(validate_type(&val, &FieldType::CreditCard).is_err());
}

#[test]
fn test_validate_type_isbn_valid() {
    let val = Value::String("9780134685991".to_string());
    assert!(validate_type(&val, &FieldType::ISBN).is_ok());
}

#[test]
fn test_validate_type_isbn_invalid() {
    let val = Value::String("not-an-isbn".to_string());
    assert!(validate_type(&val, &FieldType::ISBN).is_err());
}

#[test]
fn test_validate_type_port_valid() {
    let val = Value::String("8080".to_string());
    assert!(validate_type(&val, &FieldType::Port).is_ok());
}

#[test]
fn test_validate_type_port_invalid() {
    let val = Value::String("70000".to_string());
    assert!(validate_type(&val, &FieldType::Port).is_err());
}



#[test]
fn test_validate_type_lat_valid() {
    let val = Value::String("45.5".to_string());
    assert!(validate_type(&val, &FieldType::Lat).is_ok());
}

#[test]
fn test_validate_type_lat_invalid() {
    let val = Value::String("91".to_string());
    assert!(validate_type(&val, &FieldType::Lat).is_err());
}

#[test]
fn test_validate_type_lat_not_number() {
    let val = Value::String("abc".to_string());
    assert!(validate_type(&val, &FieldType::Lat).is_err());
}

#[test]
fn test_validate_type_lng_valid() {
    let val = Value::String("120.5".to_string());
    assert!(validate_type(&val, &FieldType::Lng).is_ok());
}

#[test]
fn test_validate_type_lng_invalid() {
    let val = Value::String("181".to_string());
    assert!(validate_type(&val, &FieldType::Lng).is_err());
}

#[test]
fn test_validate_type_semver_valid() {
    let val = Value::String("1.2.3".to_string());
    assert!(validate_type(&val, &FieldType::SemVer).is_ok());
}

#[test]
fn test_validate_type_semver_with_prerelease() {
    let val = Value::String("1.0.0-alpha".to_string());
    assert!(validate_type(&val, &FieldType::SemVer).is_ok());
}

#[test]
fn test_validate_type_semver_invalid() {
    let val = Value::String("not-semver".to_string());
    assert!(validate_type(&val, &FieldType::SemVer).is_err());
}

#[test]
fn test_validate_type_username_valid() {
    let val = Value::String("john_doe".to_string());
    assert!(validate_type(&val, &FieldType::Username).is_ok());
}

#[test]
fn test_validate_type_username_invalid() {
    let val = Value::String("ab".to_string());
    assert!(validate_type(&val, &FieldType::Username).is_err());
}

#[test]
fn test_validate_type_countrycode_valid() {
    let val = Value::String("US".to_string());
    assert!(validate_type(&val, &FieldType::CountryCode).is_ok());
}

#[test]
fn test_validate_type_countrycode_invalid() {
    let val = Value::String("USA".to_string());
    assert!(validate_type(&val, &FieldType::CountryCode).is_err());
}

#[test]
fn test_validate_type_postalcode_valid() {
    let val = Value::String("ABC123".to_string());
    assert!(validate_type(&val, &FieldType::PostalCode).is_ok());
}

#[test]
fn test_validate_type_postalcode_invalid() {
    let val = Value::String("!".to_string());
    assert!(validate_type(&val, &FieldType::PostalCode).is_err());
}

#[test]
fn test_validate_type_filepath_valid() {
    let val = Value::String("/path/to/file".to_string());
    assert!(validate_type(&val, &FieldType::FilePath).is_ok());
}

#[test]
fn test_validate_type_filepath_windows() {
    let val = Value::String("C:/path/to/file".to_string());
    assert!(validate_type(&val, &FieldType::FilePath).is_ok());
}

#[test]
fn test_validate_type_alpha_valid() {
    let val = Value::String("abcDEF".to_string());
    assert!(validate_type(&val, &FieldType::Alpha).is_ok());
}

#[test]
fn test_validate_type_alpha_invalid() {
    let val = Value::String("abc123".to_string());
    assert!(validate_type(&val, &FieldType::Alpha).is_err());
}

#[test]
fn test_validate_type_alphanumeric_valid() {
    let val = Value::String("abc123".to_string());
    assert!(validate_type(&val, &FieldType::Alphanumeric).is_ok());
}

#[test]
fn test_validate_type_alphanumeric_invalid() {
    let val = Value::String("abc-123".to_string());
    assert!(validate_type(&val, &FieldType::Alphanumeric).is_err());
}

#[test]
fn test_validate_type_custom_regex_valid() {
    let val = Value::String("ABC".to_string());
    assert!(validate_type(&val, &FieldType::Custom(r"^[A-Z]{3}$".to_string())).is_ok());
}

#[test]
fn test_validate_type_custom_regex_invalid() {
    let val = Value::String("abc".to_string());
    assert!(validate_type(&val, &FieldType::Custom(r"^[A-Z]{3}$".to_string())).is_err());
}

#[test]
fn test_validate_type_custom_regex_not_string() {
    let val = Value::Int(123);
    assert!(validate_type(&val, &FieldType::Custom(r"^[A-Z]+$".to_string())).is_err());
}

#[test]
fn test_validate_type_not_string() {
    let val = Value::Int(123);
    assert!(validate_type(&val, &FieldType::String).is_err());
}

#[test]
fn test_validate_type_not_int() {
    let val = Value::String("abc".to_string());
    assert!(validate_type(&val, &FieldType::Int).is_err());
}

#[test]
fn test_validate_type_not_float() {
    let val = Value::String("abc".to_string());
    assert!(validate_type(&val, &FieldType::Float).is_err());
}

#[test]
fn test_validate_type_not_bool() {
    let val = Value::String("abc".to_string());
    assert!(validate_type(&val, &FieldType::Bool).is_err());
}

#[test]
fn test_validate_type_not_object() {
    let val = Value::String("abc".to_string());
    assert!(validate_type(&val, &FieldType::Object).is_err());
}

#[test]
fn test_validate_type_not_array() {
    let val = Value::String("abc".to_string());
    assert!(validate_type(&val, &FieldType::Array).is_err());
}

#[test]
fn test_validate_type_not_timestamp() {
    let val = Value::String("abc".to_string());
    assert!(validate_type(&val, &FieldType::Timestamp).is_err());
}

#[test]
fn test_validate_type_password() {
    let val = Value::String("password123".to_string());
    assert!(validate_type(&val, &FieldType::Password).is_ok());
}

#[test]
fn test_validate_type_password_not_string() {
    let val = Value::Int(123);
    assert!(validate_type(&val, &FieldType::Password).is_err());
}

#[test]
fn test_validate_type_token() {
    let val = Value::String("token123".to_string());
    assert!(validate_type(&val, &FieldType::Token).is_ok());
}

#[test]
fn test_validate_type_token_not_string() {
    let val = Value::Int(123);
    assert!(validate_type(&val, &FieldType::Token).is_err());
}

#[test]
fn test_validate_type_uri_invalid() {
    let val = Value::String("not-a-uri".to_string());
    assert!(validate_type(&val, &FieldType::Uri).is_err());
}

#[test]
fn test_validate_type_hostname_empty() {
    let val = Value::String("".to_string());
    assert!(validate_type(&val, &FieldType::Hostname).is_err());
}

#[test]
fn test_validate_type_hostname_too_long() {
    let val = Value::String("a".repeat(300).to_string());
    assert!(validate_type(&val, &FieldType::Hostname).is_err());
}

#[test]
fn test_validate_type_hostname_invalid() {
    let val = Value::String("-invalid".to_string());
    assert!(validate_type(&val, &FieldType::Hostname).is_err());
}

#[test]
fn test_validate_complex_object_with_validate_function() {
    let dsl = r#"
    (
        name:string,
        age:int=0,
        tags:array<string>
    )
    "#;
    let rules = Parser::parse_rules(dsl).unwrap();
    let data = Value::Object({
        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::String("john".to_string()));
        map.insert("tags".to_string(), Value::Array(vec![Value::String("a".to_string())]));
        map
    });
    let result = validate(&data, &rules).unwrap();
    assert!(matches!(result, Value::Object(_)));
}

#[test]
fn test_validate_array_with_defaults() {
    let dsl = r#"(items:array<int[0,100]>)"#;
    let rules = Parser::parse_rules(dsl).unwrap();
    let mut items = vec![Value::Int(1), Value::Int(2)];
    let data = Value::Object({
        let mut map = HashMap::new();
        map.insert("items".to_string(), Value::Array(items));
        map
    });
    let result = validate(&data, &rules);
    assert!(result.is_some());
}

#[test]
fn test_validate_nested_object_with_validate() {
    let dsl = r#"(user:object(name:string, age:int=0))"#;
    let rules = Parser::parse_rules(dsl).unwrap();
    let mut user = HashMap::new();
    user.insert("name".to_string(), Value::String("john".to_string()));
    let data = Value::Object({
        let mut map = HashMap::new();
        map.insert("user".to_string(), Value::Object(user));
        map
    });
    let result = validate(&data, &rules);
    assert!(result.is_some());
}

#[test]
fn test_validate_object_not_object() {
    let rules = Parser::parse_rules("(name:string)").unwrap();
    let data = Value::String("not an object".to_string());
    let result = validate(&data, &rules);
    assert!(result.is_none());
}

#[test]
fn test_validate_optional_field_with_validate() {
    let dsl = r#"(name?:string)"#;
    let rules = Parser::parse_rules(dsl).unwrap();
    let data = Value::Object(HashMap::new());
    let result = validate(&data, &rules);
    assert!(result.is_some());
}

#[test]
fn test_validate_enum_with_validate() {
    let dsl = r#"(status:string enum("active","inactive"))"#;
    let rules = Parser::parse_rules(dsl).unwrap();
    let data = Value::Object({
        let mut map = HashMap::new();
        map.insert("status".to_string(), Value::String("active".to_string()));
        map
    });
    let result = validate(&data, &rules);
    assert!(result.is_some());
}

#[test]
fn test_validate_union_type_with_validate() {
    let dsl = r#"(value:int|string)"#;
    let rules = Parser::parse_rules(dsl).unwrap();
    let data = Value::Object({
        let mut map = HashMap::new();
        map.insert("value".to_string(), Value::Int(42));
        map
    });
    let result = validate(&data, &rules);
    assert!(result.is_some());
}

#[test]
fn test_validate_regex_constraint_with_validate() {
    let dsl = r#"(code:string regex("^[A-Z]{3}$"))"#;
    let rules = Parser::parse_rules(dsl).unwrap();
    let data = Value::Object({
        let mut map = HashMap::new();
        map.insert("code".to_string(), Value::String("ABC".to_string()));
        map
    });
    let result = validate(&data, &rules);
    assert!(result.is_some());
}

#[test]
fn test_validate_type_custom_invalid_regex() {
    let val = Value::String("test".to_string());
    let result = validate_type(&val, &FieldType::Custom(r"[invalid".to_string()));
    assert!(result.is_err());
    if let Err(ValidationError::InvalidRegex(_)) = result {
        assert!(true);
    } else {
        assert!(false);
    }
}

#[test]
fn test_validate_full_dsl_with_validate_function() {
    let dsl = r#"
    (
        id:uuid,
        username:string[3,20],
        email:email,
        age:int[0,150]=18,
        role:string enum("admin","user","guest")="user",
        active:bool=true
    )
    "#;
    let rules = Parser::parse_rules(dsl).unwrap();
    let data = Value::Object({
        let mut map = HashMap::new();
        map.insert("id".to_string(), Value::String("550e8400-e29b-41d4-a716-446655440000".to_string()));
        map.insert("username".to_string(), Value::String("john_doe".to_string()));
        map.insert("email".to_string(), Value::String("john@example.com".to_string()));
        map.insert("role".to_string(), Value::String("admin".to_string()));
        map.insert("active".to_string(), Value::Bool(true));
        map
    });
    let result = validate(&data, &rules);
    assert!(result.is_some());
}

#[test]
fn test_validate_fills_defaults_with_validate() {
    let dsl = r#"(name:string, age:int=25, active:bool=false)"#;
    let rules = Parser::parse_rules(dsl).unwrap();
    let data = Value::Object({
        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::String("john".to_string()));
        map
    });
    let result = validate(&data, &rules).unwrap();
    if let Value::Object(obj) = result {
        assert_eq!(obj.get("age"), Some(&Value::Int(25)));
        assert_eq!(obj.get("active"), Some(&Value::Bool(false)));
    } else {
        assert!(false);
    }
}

#[test]
fn test_validate_ip_v6() {
    let val = Value::String("2001:0db8:85a3:0000:0000:8a2e:0370:7334".to_string());
    assert!(validate_type(&val, &FieldType::Ip).is_ok());
}

#[test]
fn test_validate_mac() {
    let val = Value::String("AA:BB:CC:DD:EE:FF".to_string());
    assert!(validate_type(&val, &FieldType::Mac).is_ok());
}

#[test]
fn test_validate_type_mac_invalid() {
    let val = Value::String("invalid-mac".to_string());
    assert!(validate_type(&val, &FieldType::Mac).is_err());
}
