# ZZ-Validator

<p align="center">
  <strong>A powerful Rust library for DSL-based data validation</strong>
</p>

<p align="center">
  <a href="https://github.com/Free-Web-Movement/validator">
    <img src="https://img.shields.io/badge/license-GPL--3.0-blue.svg" alt="License">
  </a>
  <a href="https://crates.io/crates/zz-validator">
    <img src="https://img.shields.io/crates/v/zz-validator.svg" alt="Crates.io">
  </a>
</p>

---

## Overview

ZZ-Validator is a lightweight Domain-Specific Language (DSL) for defining validation rules for JSON-like data structures. It provides an elegant way to describe field types, constraints, default values, and optional fields.

### Features

- **Type System**: Rich built-in types + custom regex support
- **Constraints**: Range, regex patterns, enum values
- **Default Values**: Automatic default value injection
- **Optional Fields**: Mark fields as optional with `?`
- **Nested Structures**: Support for complex objects and arrays
- **Union Types**: Multiple acceptable types per field
- **High Performance**: Pre-compiled regex, regex caching

---

## Quick Start

### Installation

```toml
# Cargo.toml
[dependencies]
zz-validator = "0.1"
```

### Basic Usage

```rust
use zz_validator::{parser::Parser, validator::validate};

// Define validation rules using DSL
let dsl = r#"
(
    username:string[3,20],
    age:int[0,150]=18,
    email?:email,
    active:bool=true
)
"#;

// Parse DSL into rules
let rules = Parser::parse_rules(dsl).unwrap();

// Validate data (doesn't modify original, returns None on failure)
use zz_validator::ast::Value;
use std::collections::HashMap;

let data = Value::Object({
    let mut map = HashMap::new();
    map.insert("username".to_string(), Value::String("john".to_string()));
    map.insert("email".to_string(), Value::String("john@example.com".to_string()));
    map
});

let validated = validate(&data, &rules).unwrap();
// validated = { username: "john", age: 18, email: "john@example.com", active: true }
```

---

## DSL Syntax

The DSL uses a simple, intuitive syntax:

```
name?:type[constraints] = default_value
```

| Component | Description |
|-----------|-------------|
| `name` | Field name |
| `?` | Makes field optional |
| `type` | Data type |
| `[constraints]` | Optional constraints |
| `= value` | Default value |

---

## Type Reference

### Basic Types

| Type | Description | Example |
|------|-------------|---------|
| `string` | UTF-8 string | `"hello"` |
| `int` | 64-bit integer | `42` |
| `float` | 64-bit float | `3.14` |
| `bool` | Boolean | `true` / `false` |
| `object` | Nested object | `{...}` |
| `array<T>` | Array of type T | `[...]` |

### Extended Types

#### Web & Network

| Type | Description | Validation |
|------|-------------|------------|
| `email` | Email address | RFC-compliant |
| `uri` | URL/URI | Valid URL format |
| `uuid` | UUID v4 | Standard UUID format |
| `ip` | IP Address | IPv4 or IPv6 |
| `mac` | MAC Address | Standard MAC format |
| `hostname` | Domain hostname | RFC-compliant |
| `urlencoded` | URL encoded string | Percent-encoding |

#### Numbers & IDs

| Type | Description | Validation |
|------|-------------|------------|
| `port` | TCP/UDP port | 0-65535 |
| `lat` | Latitude | -90 to 90 |
| `lng` | Longitude | -180 to 180 |

#### Date & Time

| Type | Description | Format |
|------|-------------|--------|
| `date` | Date | YYYY-MM-DD |
| `time` | Time | HH:MM:SS |
| `datetime` | DateTime | ISO8601 |
| `timestamp` | Unix timestamp | Numeric |

#### Format & Encoding

| Type | Description | Example |
|------|-------------|---------|
| `json` | JSON string | Valid JSON |
| `hex` | Hexadecimal | `deadbeef` |
| `base64` | Base64 encoded | Standard Base64 |

#### Identifiers

| Type | Description | Pattern |
|------|-------------|---------|
| `slug` | URL slug | `my-post-title` |
| `username` | Username | 3-20 chars, alphanumeric |
| `countrycode` | Country code | ISO 3166-1 alpha-2 |
| `postalcode` | Postal code | 3-10 alphanumeric |
| `semver` | Semantic version | `1.0.0` |
| `isbn` | ISBN (10/13) | Standard ISBN |

#### Content Types

| Type | Description | Pattern |
|------|-------------|---------|
| `alpha` | Alphabetic only | `[a-zA-Z]+` |
| `alphanumeric` | Alphanumeric | `[a-zA-Z0-9]+` |
| `color` | Hex color | `#RGB` or `#RRGGBB` |
| `phone` | Phone number | E.164 format |
| `creditcard` | Credit card | Luhn algorithm |
| `filepath` | File path | Platform-aware |

#### Security

| Type | Description |
|------|-------------|
| `password` | Password string |
| `token` | Auth token |

### Custom Regex Type

Define your own validation pattern:

```dsl
code:regex("^[A-Z]{3}$")
phone:regex("^\\+86[1-9]\\d{10}$")
chinese:regex("^[\\u4e00-\\u9fa5]+$")
```

---

## Constraints

### Range Constraint

**For numbers** (`int`, `float`):
```dsl
age:int[0,150]          // inclusive: 0 <= age <= 150
score:float(0,100)      // exclusive: 0 < score < 100
```

**For strings** (length):
```dsl
username:string[3,20]  // length: 3 <= len <= 20
```

### Regex Constraint

```dsl
username:string regex("^[a-zA-Z0-9_]+$")
```

### Enum Constraint

```dsl
role:string enum("admin","user","guest")
status:string enum("active","inactive","pending")
```

---

## Default Values

Provide fallback values for missing fields:

```dsl
age:int[0,150]=18
active:bool=true
name:string="Anonymous"
score:float=0.0
```

---

## Optional Fields

Mark fields as optional with `?`:

```dsl
nickname?:string[0,20]    // field can be missing
email?:email              // optional email
phone?:string             // optional phone
```

---

## Union Types

A field can accept multiple types:

```dsl
id:int|float             // accepts both int and float
value:int|string|bool    // accepts multiple types
```

---

## Nested Objects

Define complex structures:

```dsl
profile:object(
    name:string[1,50],
    age:int[0,150],
    contact:object(
        email:email,
        phone?:string
    )
)
```

---

## Arrays

Validate array elements:

```dsl
tags:array<string[1,10]>
scores:array<int[0,100]>
users:array<object(
    name:string,
    email:email
)>
```

---

## API Reference

### Parser

```rust
use zz_validator::parser::Parser;

// Parse DSL string into rules
let rules = Parser::parse_rules(dsl).unwrap();
```

### Validator

```rust
use zz_validator::validator::{validate_object, validate};

// Method 1: In-place validation (modifies the input Value, fills defaults)
let result = validate_object(&mut value, &rules);

// Method 2: Validate and return a new Value (recommended, returns None on failure)
let validated = validate(&value, &rules);
// validated is Option<Value>, contains the new Value with defaults on success
```

### ValidationError

```rust
use zz_validator::validator::ValidationError;

match error {
    ValidationError::MissingField(f) => ...,
    ValidationError::TypeMismatch { field, expected, actual } => ...,
    ValidationError::RangeError { field, value, min, max } => ...,
    ValidationError::EnumMismatch { field, value, expected } => ...,
    ValidationError::RegexMismatch { field, pattern } => ...,
    ValidationError::Custom(msg) => ...,
}
```

---

## Complete Example

```dsl
(
    id:uuid,
    username:string[3,20] regex("^[a-zA-Z0-9_]+$"),
    email:email,
    age:int[0,150]=18,
    role:string enum("admin","user","guest")="user",
    active:bool=true,
    profile:object(
        first_name:string[1,50],
        last_name:string[1,50],
        contact:object(
            email:email,
            phone?:phone
        )
    ),
    tags:array<string[1,10]>,
    scores:array<int[0,100]>,
    version:semver,
    location:object(
        lat:lat,
        lng:lng
    )
)
```

---

## Error Messages

ZZ-Validator provides detailed error messages:

```
Missing required field 'username'
username value "ab": expected string length >= 3
age value 200 out of range [0, 150]
role value "superadmin" not in enum ["admin","user","guest"]
email value "invalid": Invalid email: invalid
```

---

## Performance

- **Regex Caching**: Custom regexes are compiled once and cached
- **Pre-compiled Patterns**: Built-in types use pre-compiled regex
- **Zero-copy**: Tokenizer minimizes allocations

---

## License

GPL-3.0-or-later - See [LICENSE](LICENSE) for details.

---

## Contributing

Contributions are welcome! Please open an issue or submit a PR on [GitHub](https://github.com/Free-Web-Movement/validator).
