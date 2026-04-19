# cmdreg

English | [中文](README_zh.md)

A lightweight, string-keyed command dispatcher for Rust with **axum-style extractors** and optional **auto-registration macros**.

Designed for applications that dispatch operations by name — such as IPC bridges, plugin systems, FFI layers, or embedded scripting hosts.

## Features

- **String-keyed dispatch** — register and invoke handlers by name (`"fs.read"`, `"workspace.open"`)
- **Axum-style extractors** — use `Json(args)` to automatically deserialize arguments from JSON
- **Sync & async handlers** — first-class support for both, in a unified API
- **`#[command]` macro** — annotate functions to auto-register them (opt-in via `macros` feature)
- **Callback commands** — register and invoke callback-style handlers
- **Zero boilerplate** — no traits to implement on your handler functions

## Quick Start

```toml
[dependencies]
cmdreg = "0.1"
anyhow = "1.0"
```

### Register & invoke commands manually

```rust
use cmdreg::{reg_command, reg_command_async, invoke_command, invoke_command_async, CommandContext};

// Sync handler
fn hello() -> cmdreg::CommandResult {
    cmdreg::CommandResponse::json("hello world")
}

reg_command("app.hello", hello).unwrap();

let result = invoke_command("app.hello", CommandContext::None).unwrap();

// Async handler
async fn greet(cmdreg::Json(name): cmdreg::Json<String>) -> cmdreg::CommandResult {
    cmdreg::CommandResponse::json(format!("hello, {}", name))
}

reg_command_async("app.greet", greet).unwrap();
```

### Auto-registration with `#[command]` macro

Enable the `macros` feature:

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

// At startup:
fn main() {
    cmdreg::reg_all_commands().unwrap();
    // "fs.exists" and "fs.read_file" are now registered
}
```

## Feature Flags

| Feature    | Default | Description                                          |
|------------|---------|------------------------------------------------------|
| `macros`   | off     | Enables `#[command]` macro and `reg_all_commands()`  |
| `full`     | off     | Enables all optional features                        |

## How It Works

1. **`CommandMap<K, F>`** — a generic `HashMap` wrapper keyed by string-like types.
2. **Global registries** — `LazyLock<Arc<RwLock<...>>>` singletons for sync, async, and callback commands.
3. **Handler traits** — `CommandHandler<T>` (sync) and `CommandHandlerAsync<T>` (async) are auto-implemented for functions with up to 10 extractor parameters.
4. **Extractors** — `Json<T>` deserializes `CommandContext` into your typed arguments, similar to axum's extractor pattern.
5. **`#[command("prefix")]`** — a proc-macro that generates a registration function and submits it to `inventory` for collection at link time.

## Requirements

- Rust nightly (uses `closure_lifetime_binder`)
- tokio runtime (for async command dispatch)

## License

[MIT](LICENSE)
