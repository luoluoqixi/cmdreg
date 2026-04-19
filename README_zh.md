# cmdreg

[English](README.md) | 中文

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
cmdreg = "0.1"
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

// 不传前缀 — 注册为 "ping"
#[command]
fn ping() -> CommandResult {
    CommandResponse::json("pong")
}

// 启动时调用：
fn main() {
    cmdreg::reg_all_commands().unwrap();
    // "fs.exists"、"fs.read_file" 和 "ping" 已自动注册
}
```

## Feature Flags

| Feature  | 默认 | 说明                                        |
| -------- | ---- | ------------------------------------------- |
| `macros` | 关闭 | 启用 `#[command]` 宏和 `reg_all_commands()` |
| `full`   | 关闭 | 启用所有可选 feature                        |

## 工作原理

1. **`CommandMap<K, F>`** — 基于字符串键的泛型 `HashMap` 封装。
2. **全局注册表** — 使用 `LazyLock<Arc<RwLock<...>>>` 单例分别管理同步、异步和回调命令。
3. **Handler trait** — `CommandHandler<T>`（同步）和 `CommandHandlerAsync<T>`（异步）为最多 10 个提取器参数的函数自动实现。
4. **提取器** — `Json<T>` 从 `CommandContext` 中反序列化出类型化参数，类似 axum 的提取器模式。
5. **`#[command("prefix")]` / `#[command]`** — proc-macro 生成注册函数，并通过 `inventory` 在链接期自动收集。不传前缀时，直接使用函数名作为命令键。

## 环境要求

- Rust nightly（使用了 `closure_lifetime_binder`）
- tokio 运行时（异步命令分发需要）

## 许可证

[MIT](LICENSE)
