# ZZ-Validator DSL 使用说明书

## 1. 简介

ZZ-Validator DSL 是一种轻量级的描述语言，用于定义 JSON/对象字段的类型、约束、默认值及可选性。  
主要用途包括：

- 字段类型验证（string、int、float、bool、object、array 等）
- 约束规则（长度、范围、正则、枚举）
- 默认值填充与可选字段支持
- 嵌套对象与数组验证
- 支持扩展类型（email、uuid、uri 等）

---

## 2. 基本语法

字段定义格式：

```dsl
<name>:<type>[<constraints>] = <default> ?
```

### 各部分含义

- `name`：字段名
- `type`：字段类型
- `[constraints]`：可选约束，如长度、范围、正则
- `= <default>`：可选默认值
- `?`：可选字段

---

## 3. 支持类型

### 3.1 基本类型

| 类型       | 描述                    |
|------------|-------------------------|
| string     | 字符串                  |
| int        | 整数                    |
| float      | 浮点数                  |
| bool       | 布尔值                  |
| object     | 对象，内部可定义子字段   |
| array      | 数组，可指定元素类型     |

### 3.2 扩展类型

| 类型       | 描述                                   |
|------------|----------------------------------------|
| email      | 邮箱地址                               |
| uri        | URL/URI                                |
| uuid       | UUID 格式                              |
| ip         | IP 地址（IPv4 或 IPv6）               |
| date       | 日期，格式 YYYY-MM-DD                   |
| datetime   | 日期时间，ISO8601 格式                  |

### 3.3 联合类型

可使用 `|` 定义多个可接受类型：

```dsl
id: int|float
```

---

## 4. 约束规则

### 4.1 长度/范围

- 字符串长度限制：`string[3,20]`
- 数值范围限制：`int[0,100]` 或 `float(0.0, 10.0)`

### 4.2 正则表达式

```dsl
username:string[3,20] regex("^[a-zA-Z0-9_]+$")
```

### 4.3 枚举值

```dsl
role:string enum("admin","user","guest")
```

### 4.4 默认值

```dsl
age:int[0,150]=30
active:bool=true
```

### 4.5 可选字段

在字段名后加 `?`：

```dsl
nickname?:string[0,20]
email?:email
```

---

## 5. 嵌套对象和数组

### 5.1 对象

```dsl
profile:object(
    first_name:string[1,50],
    last_name:string[1,50],
    contact:object(
        email:email,
        phone?:string[0,20]
    )
)
```

### 5.2 数组

```dsl
tags:array<string[1,10]>
```

---

## 6. 示例 DSL

```dsl
(
    username:string[3,20] regex("^[a-zA-Z0-9_]+$"),
    age:int[0,150]=30,
    email?:email,
    id:uuid,
    homepage?:uri,
    profile:object(
        first_name:string[1,50],
        last_name:string[1,50],
        contact:object(
            email:email,
            phone?:string[0,20]
        )
    ),
    tags:array<string[1,10]>,
    distance:float[1.0,2.0]=1.5
)
```

---

## 7. 验证规则说明

- **默认值**：当字段缺失且有默认值时自动填充
- **可选字段**：字段缺失不会报错
- **union types**：值需匹配至少一个类型
- **嵌套对象/数组**：递归验证所有子字段
- **正则与枚举**：严格匹配指定模式或枚举值
- **范围约束**：适用于 int、float 和 string（长度）类型

---

## 8. 扩展指南

- **增加自定义类型**：
  - 如 mac_address、isbn、credit_card 等
  - 可通过正则或回调函数实现
- **扩展验证规则**：
  - 自定义 constraint 类型
  - 嵌套对象可递归使用新类型

---

## 9. 注意事项

- 字符串长度用 `usize` 计算
- union types 验证顺序从左到右，匹配即停止
- 对象默认值需通过 `Value::Object` 插入
- 数组验证对每个元素执行规则
