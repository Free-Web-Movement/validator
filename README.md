# ZZ-Validator

<p align="center">
  <strong>强大的 Rust DSL 数据验证库</strong>
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

## 简介

ZZ-Validator 是一个轻量级的领域特定语言（DSL），用于为 JSON 数据结构定义验证规则。它提供了一种优雅的方式来描述字段类型、约束、默认值和可选字段。

### 特性

- **类型系统**: 丰富的内置类型 + 自定义正则支持
- **约束**: 范围、正则模式、枚举值
- **默认值**: 自动注入默认值
- **可选字段**: 使用 `?` 标记可选字段
- **嵌套结构**: 支持复杂的对象和数组
- **联合类型**: 字段可接受多种类型
- **高性能**: 预编译正则，正则缓存

---

## 快速开始

### 安装

```toml
# Cargo.toml
[dependencies]
zz-validator = "0.1"
```

### 基本用法

```rust
use zz_validator::{parser::Parser, validator::validate};

// 使用 DSL 定义验证规则
let dsl = r#"
(
    username:string[3,20],
    age:int[0,150]=18,
    email?:email,
    active:bool=true
)
"#;

// 将 DSL 解析为规则
let rules = Parser::parse_rules(dsl).unwrap();

// 验证数据（不修改原数据，失败返回 None）
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

## DSL 语法

DSL 使用简洁直观的语法：

```
name?:type[constraints] = default_value
```

| 组件 | 描述 |
|------|------|
| `name` | 字段名 |
| `?` | 标记字段为可选 |
| `type` | 数据类型 |
| `[constraints]` | 可选约束 |
| `= value` | 默认值 |

---

## 类型参考

### 基本类型

| 类型 | 描述 | 示例 |
|------|------|------|
| `string` | UTF-8 字符串 | `"hello"` |
| `int` | 64位整数 | `42` |
| `float` | 64位浮点数 | `3.14` |
| `bool` | 布尔值 | `true` / `false` |
| `object` | 嵌套对象 | `{...}` |
| `array<T>` | T 类型的数组 | `[...]` |

### 扩展类型

#### Web 与网络

| 类型 | 描述 | 验证规则 |
|------|------|----------|
| `email` | 邮箱地址 | RFC 规范 |
| `uri` | URL/URI | 有效 URL 格式 |
| `uuid` | UUID | 标准 UUID 格式 |
| `ip` | IP 地址 | IPv4 或 IPv6 |
| `mac` | MAC 地址 | 标准 MAC 格式 |
| `hostname` | 域名主机名 | RFC 规范 |
| `urlencoded` | URL 编码字符串 | 百分号编码 |

#### 数字与 ID

| 类型 | 描述 | 验证规则 |
|------|------|----------|
| `port` | TCP/UDP 端口 | 0-65535 |
| `lat` | 纬度 | -90 到 90 |
| `lng` | 经度 | -180 到 180 |

#### 日期与时间

| 类型 | 描述 | 格式 |
|------|------|------|
| `date` | 日期 | YYYY-MM-DD |
| `time` | 时间 | HH:MM:SS |
| `datetime` | 日期时间 | ISO8601 |
| `timestamp` | Unix 时间戳 | 数字 |

#### 格式与编码

| 类型 | 描述 | 示例 |
|------|------|------|
| `json` | JSON 字符串 | 有效 JSON |
| `hex` | 十六进制 | `deadbeef` |
| `base64` | Base64 编码 | 标准 Base64 |

#### 标识符

| 类型 | 描述 | 模式 |
|------|------|------|
| `slug` | URL  slug | `my-post-title` |
| `username` | 用户名 | 3-20 字符，字母数字 |
| `countrycode` | 国家代码 | ISO 3166-1 alpha-2 |
| `postalcode` | 邮政编码 | 3-10 字母数字 |
| `semver` | 语义化版本 | `1.0.0` |
| `isbn` | ISBN (10/13) | 标准 ISBN |

#### 内容类型

| 类型 | 描述 | 模式 |
|------|------|------|
| `alpha` | 仅字母 | `[a-zA-Z]+` |
| `alphanumeric` | 字母数字 | `[a-zA-Z0-9]+` |
| `color` | 十六进制颜色 | `#RGB` 或 `#RRGGBB` |
| `phone` | 电话号码 | E.164 格式 |
| `creditcard` | 信用卡号 | Luhn 算法 |
| `filepath` | 文件路径 | 平台感知 |

#### 安全相关

| 类型 | 描述 |
|------|------|
| `password` | 密码字符串 |
| `token` | 认证令牌 |

### 自定义正则类型

定义你自己的验证模式：

```dsl
code:regex("^[A-Z]{3}$")
phone:regex("^\\+86[1-9]\\d{10}$")
chinese:regex("^[\\u4e00-\\u9fa5]+$")
```

---

## 约束

### 范围约束

**对于数字** (`int`, `float`):
```dsl
age:int[0,150]          // 包含: 0 <= age <= 150
score:float(0,100)      // 排除: 0 < score < 100
```

**对于字符串** (长度):
```dsl
username:string[3,20]  // 长度: 3 <= len <= 20
```

### 正则约束

```dsl
username:string regex("^[a-zA-Z0-9_]+$")
```

### 枚举约束

```dsl
role:string enum("admin","user","guest")
status:string enum("active","inactive","pending")
```

---

## 默认值

为缺失字段提供后备值：

```dsl
age:int[0,150]=18
active:bool=true
name:string="Anonymous"
score:float=0.0
```

---

## 可选字段

使用 `?` 标记字段为可选：

```dsl
nickname?:string[0,20]    // 字段可以缺失
email?:email              // 可选邮箱
phone?:string             // 可选电话
```

---

## 联合类型

一个字段可以接受多种类型：

```dsl
id:int|float             // 同时接受 int 和 float
value:int|string|bool    // 接受多种类型
```

---

## 嵌套对象

定义复杂结构：

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

## 数组

验证数组元素：

```dsl
tags:array<string[1,10]>
scores:array<int[0,100]>
users:array<object(
    name:string,
    email:email
)>
```

---

## API 参考

### Parser

```rust
use zz_validator::parser::Parser;

// 将 DSL 字符串解析为规则
let rules = Parser::parse_rules(dsl).unwrap();
```

### Validator

```rust
use zz_validator::validator::{validate_object, validate};

// 方式一：原地验证（会修改传入的 Value，填充默认值）
let result = validate_object(&mut value, &rules);

// 方式二：验证并返回新对象（推荐，不修改原数据，失败返回 None）
let validated = validate(&value, &rules);
// validated 是 Option<Value>，验证通过时包含带默认值的新 Value
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

## 完整示例

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

## 错误信息

ZZ-Validator 提供详细的错误信息：

```
Missing required field 'username'
username value "ab": expected string length >= 3
age value 200 out of range [0, 150]
role value "superadmin" not in enum ["admin","user","guest"]
email value "invalid": Invalid email: invalid
```

---

## 性能

- **正则缓存**: 自定义正则只编译一次并缓存
- **预编译模式**: 内置类型使用预编译正则
- **零拷贝**: Tokenizer 最小化内存分配

---

## 许可证

GPL-3.0-or-later - 详见 [LICENSE](LICENSE)。

---

## 贡献

欢迎贡献！请在 [GitHub](https://github.com/Free-Web-Movement/validator) 上提交 issue 或 PR。
