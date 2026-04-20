# cmdreg

[![Crates.io](https://img.shields.io/crates/v/cmdreg.svg)](https://crates.io/crates/cmdreg)
[![docs.rs](https://docs.rs/cmdreg/badge.svg)](https://docs.rs/cmdreg)
[![CI](https://github.com/luoluoqixi/cmdreg/actions/workflows/ci.yml/badge.svg)](https://github.com/luoluoqixi/cmdreg/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/crates/l/cmdreg.svg)](LICENSE)

English | [中文](https://github.com/luoluoqixi/cmdreg/blob/main/README_zh.md)

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
cmdreg = "0.2"
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

#### Classic style (extractor-based)

Use `Json<T>` extractors and return `CommandResult` explicitly:

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

#### Plain style (auto-generated)

Use plain parameters and any `Serialize` return type — the macro auto-generates
a `#[derive(Deserialize)]` args struct and wraps the return value with
`CommandResponse::json()`:

```rust
use cmdreg::command;

#[command("fs")]
fn get_file_list(path: String, recursive: bool) -> Vec<String> {
    // caller passes: {"path": "./src", "recursive": true}
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

Supported return types in plain style:

| Return type            | Behavior                                    |
| ---------------------- | ------------------------------------------- |
| `T: Serialize`         | Wrapped with `CommandResponse::json(value)` |
| `Result<T: Serialize>` | Unwrapped with `?`, then wrapped with json  |
| `CommandResult`        | Passed through directly                     |
| `()` / no return       | Returns `Ok(CommandResponse::None)`         |

> **Note:** Plain-style parameters must be owned types (e.g. `String`, not `&str`).
> Reference types cannot be deserialized from JSON and will produce a compile error.

```rust
// At startup:
fn main() {
    cmdreg::reg_all_commands().unwrap();
    // "fs.get_file_list", "math.divide", "get_version", etc. are registered
}
```

### Global Configuration

Set a default `rename_all` for all `#[command]` macros in your crate's `Cargo.toml`:

```toml
[package.metadata.cmdreg]
rename_all = "camelCase"
```

The proc-macro reads `Cargo.toml` at compile time. Without this config, field names
match Rust parameter names as-is.

Per-function `rename_all` overrides the global default:

```rust
#[command("fs", rename_all = "camelCase")]
fn get_file_list(file_path: String, is_recursive: bool) -> Vec<String> {
    // JSON: {"filePath": "...", "isRecursive": true}
    vec![]
}

#[command("stats", rename_all = "SCREAMING_SNAKE_CASE")]
fn get_total_count(my_value: i32) -> i32 {
    // JSON: {"MY_VALUE": 42} — overrides global default
    my_value
}
```

### Command Metadata Export

Enable the `metadata` feature to export registered command metadata as JSON — useful for codegen, documentation, or client SDK generation:

```toml
[dependencies]
cmdreg = { version = "0.2", features = ["metadata"] }
```

```rust
use cmdreg::{reg_all_commands, export_commands_json, get_all_command_metas};

fn main() {
    reg_all_commands().unwrap();

    // Export to a JSON file
    export_commands_json("commands.json").unwrap();

    // Or access metadata programmatically
    for meta in get_all_command_metas() {
        println!("{} ({})", meta.name, meta.style);
    }
}
```

Generated JSON:

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

| Feature    | Default | Description                                           |
| ---------- | ------- | ----------------------------------------------------- |
| `macros`   | off     | Enables `#[command]` macro and `reg_all_commands()`   |
| `metadata` | off     | Enables command metadata export (requires `macros`)   |
| `full`     | off     | Enables all optional features (`macros` + `metadata`) |

## How It Works

1. **`CommandMap<K, F>`** — a generic `HashMap` wrapper keyed by string-like types.
2. **Global registries** — `LazyLock<Arc<RwLock<...>>>` singletons for sync, async, and callback commands.
3. **Handler traits** — `CommandHandler<T>` (sync) and `CommandHandlerAsync<T>` (async) are auto-implemented for functions with up to 10 extractor parameters.
4. **Extractors** — `Json<T>` deserializes `CommandContext` into your typed arguments, similar to axum's extractor pattern.
5. **`#[command("prefix")]` / `#[command]`** — a proc-macro that generates a registration function and submits it to `inventory` for collection at link time. When no prefix is given, the function name is used directly as the command key.
6. **Plain-style support** — when using plain parameters (e.g. `path: String`) instead of `Json<T>` extractors, the macro auto-generates a `#[derive(Deserialize)]` args struct and wraps the return value as JSON.

## Requirements

- Rust nightly (uses `closure_lifetime_binder`)
- tokio runtime (for async command dispatch)

## License

[MIT](LICENSE)
