# cmdreg

[![Crates.io](https://img.shields.io/crates/v/cmdreg.svg)](https://crates.io/crates/cmdreg)
[![docs.rs](https://docs.rs/cmdreg/badge.svg)](https://docs.rs/cmdreg)
[![CI](https://github.com/luoluoqixi/cmdreg/actions/workflows/ci.yml/badge.svg)](https://github.com/luoluoqixi/cmdreg/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/crates/l/cmdreg.svg)](LICENSE)

[English](https://github.com/luoluoqixi/cmdreg/blob/main/README.md) | 中文

一个轻量级的、基于字符串键的 Rust 命令分发框架，支持 **axum 风格的参数提取器** 和可选的 **自动注册宏**。

适用于按名称分发操作的场景 —— 如 IPC 桥接、插件系统、FFI 层、内嵌脚本宿主等。

## 特性

- **字符串键分发** — 按名称注册和调用 handler（如 `"fs.read"`、`"workspace.open"`）
- **axum 风格提取器** — 使用 `Json(args)` 自动从 JSON 反序列化参数
- **同步 & 异步 handler** — 统一 API，一等支持
- **`#[command]` 宏** — 标注函数即自动注册（通过 `macros` feature 启用）
- **回调命令** — 注册和调用回调风格的 handler
- **零模板代码** — handler 函数无需实现任何 trait

## 快速开始

```toml
[dependencies]
cmdreg = "0.2"
anyhow = "1.0"
```

### 手动注册 & 调用命令

```rust
use cmdreg::{reg_command, reg_command_async, invoke_command, invoke_command_async, CommandContext};

// 同步 handler
fn hello() -> cmdreg::CommandResult {
    cmdreg::CommandResponse::json("hello world")
}

reg_command("app.hello", hello).unwrap();

let result = invoke_command("app.hello", CommandContext::None).unwrap();

// 异步 handler
async fn greet(cmdreg::Json(name): cmdreg::Json<String>) -> cmdreg::CommandResult {
    cmdreg::CommandResponse::json(format!("hello, {}", name))
}

reg_command_async("app.greet", greet).unwrap();
```

### 使用 `#[command]` 宏自动注册

启用 `macros` feature：

```toml
[dependencies]
cmdreg = { version = "0.1", features = ["macros"] }
```

#### 经典风格（提取器）

使用 `Json<T>` 提取器，显式返回 `CommandResult`：

```rust
use cmdreg::{command, Json, CommandResult, CommandResponse};

#[command("fs")]
fn exists(Json(path): Json<String>) -> CommandResult {
    let exists = std::path::Path::new(&path).exists();
    CommandResponse::json(exists)
}

#[command("fs")]
async fn read_file(Json(path): Json<String>) -> CommandResult {
    let content = tokio::fs::read_to_string(&path).await?;
    CommandResponse::json(content)
}
```

#### 简洁风格（自动生成）

使用普通参数和任意 `Serialize` 返回类型 — 宏会自动生成 `#[derive(Deserialize)]`
参数结构体，并用 `CommandResponse::json()` 包裹返回值：

```rust
use cmdreg::command;

#[command("fs")]
fn get_file_list(path: String, recursive: bool) -> Vec<String> {
    // 调用方传参: {"path": "./src", "recursive": true}
    vec![]
}

#[command("math")]
fn divide(a: f64, b: f64) -> anyhow::Result<f64> {
    if b == 0.0 { anyhow::bail!("division by zero"); }
    Ok(a / b)
}

#[command]
fn get_version() -> String {
    "1.0.0".to_string()
}
```

简洁风格支持的返回类型：

| 返回类型               | 行为                                   |
| ---------------------- | -------------------------------------- |
| `T: Serialize`         | 用 `CommandResponse::json(value)` 包裹 |
| `Result<T: Serialize>` | 先用 `?` 解包，再用 json 包裹          |
| `CommandResult`        | 直接透传                               |
| `()` / 无返回值        | 返回 `Ok(CommandResponse::None)`       |

> **注意：** 简洁风格的参数必须是拥有所有权的类型（如 `String`，不能用 `&str`）。
> 引用类型无法从 JSON 反序列化，会产生编译错误。

```rust
// 启动时调用：
fn main() {
    cmdreg::reg_all_commands().unwrap();
    // "fs.get_file_list"、"math.divide"、"get_version" 等已自动注册
}
```

### 全局配置

在 `Cargo.toml` 中为所有 `#[command]` 设置默认的 `rename_all`：

```toml
[package.metadata.cmdreg]
rename_all = "camelCase"
```

proc-macro 在编译期读取 `Cargo.toml`。未配置时字段名保持与 Rust 参数名一致。

也可在单个函数上用 `rename_all` 覆盖全局默认：

```rust
#[command("fs", rename_all = "camelCase")]
fn get_file_list(file_path: String, is_recursive: bool) -> Vec<String> {
    // JSON: {"filePath": "...", "isRecursive": true}
    vec![]
}

#[command("stats", rename_all = "SCREAMING_SNAKE_CASE")]
fn get_total_count(my_value: i32) -> i32 {
    // JSON: {"MY_VALUE": 42}  — 覆盖全局默认
    my_value
}
```

### 命令元数据导出

启用 `metadata` feature 可将已注册命令的元数据导出为 JSON —— 适用于代码生成、文档生成或客户端 SDK 生成等场景：

```toml
[dependencies]
cmdreg = { version = "0.2", features = ["metadata"] }
```

```rust
use cmdreg::{reg_all_commands, export_commands_json, get_all_command_metas};

fn main() {
    reg_all_commands().unwrap();

    // 导出到 JSON 文件
    export_commands_json("commands.json").unwrap();

    // 或以编程方式访问元数据
    for meta in get_all_command_metas() {
        println!("{} ({})", meta.name, meta.style);
    }
}
```

生成的 JSON：

```json
[
  {
    "name": "fs.read",
    "is_async": true,
    "style": "plain",
    "params": [
      { "name": "path", "type": "String" },
      { "name": "encoding", "type": "String" }
    ],
    "return_type": "Result < String >"
  }
]
```

## Feature Flags

| Feature    | 默认 | 说明                                          |
| ---------- | ---- | --------------------------------------------- |
| `macros`   | 关闭 | 启用 `#[command]` 宏和 `reg_all_commands()`   |
| `metadata` | 关闭 | 启用命令元数据导出（依赖 `macros`）           |
| `full`     | 关闭 | 启用所有可选 feature（`macros` + `metadata`） |

## 工作原理

1. **`CommandMap<K, F>`** — 基于字符串键的泛型 `HashMap` 封装。
2. **全局注册表** — 使用 `LazyLock<Arc<RwLock<...>>>` 单例分别管理同步、异步和回调命令。
3. **Handler trait** — `CommandHandler<T>`（同步）和 `CommandHandlerAsync<T>`（异步）为最多 10 个提取器参数的函数自动实现。
4. **提取器** — `Json<T>` 从 `CommandContext` 中反序列化出类型化参数，类似 axum 的提取器模式。
5. **`#[command("prefix")]` / `#[command]`** — proc-macro 生成注册函数，并通过 `inventory` 在链接期自动收集。不传前缀时，直接使用函数名作为命令键。
6. **简洁风格支持** — 使用普通参数（如 `path: String`）代替 `Json<T>` 提取器时，宏自动生成 `#[derive(Deserialize)]` 参数结构体并将返回值包裹为 JSON。

## 环境要求

- Rust nightly（使用了 `closure_lifetime_binder`）
- tokio 运行时（异步命令分发需要）

## 许可证

[MIT](LICENSE)
