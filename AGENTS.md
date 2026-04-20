# AGENTS.md

## Project Overview

**cmdreg** is a lightweight, string-keyed command dispatch framework for Rust, featuring axum-style parameter extractors and optional auto-registration macros. It targets scenarios like IPC bridges, plugin systems, FFI layers, and embedded script hosts.

## Repository Structure

```
cmdreg/                  # Workspace root
├── Cargo.toml           # Workspace manifest (resolver = 2)
├── cmdreg/              # Core library crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs               # Module declarations and public re-exports
│       ├── command_map.rs       # Generic HashMap wrapper for handler storage
│       ├── context.rs           # CommandContext enum (String/Value/None)
│       ├── handler.rs           # Sync handler trait (CommandHandler<T>)
│       ├── handler_async.rs     # Async handler trait (CommandHandlerAsync<T>)
│       ├── dispatch_sync.rs     # Global sync command registry and dispatch
│       ├── dispatch_async.rs    # Global async command registry and dispatch
│       ├── extractor.rs         # Json<T> extractor and FromCommandArgs trait
│       ├── response.rs          # CommandResponse and CommandResult types
│       ├── callback.rs          # Callback-style handler registry
│       └── registry.rs          # Macro-based registration collector (macros feature)
└── cmdreg-macros/       # Procedural macro crate
    ├── Cargo.toml
    └── src/
        └── lib.rs               # #[command("prefix")] attribute macro
```

## Build & Toolchain

- **Language**: Rust (Edition 2021)
- **Toolchain**: Nightly required (`#![feature(closure_lifetime_binder)]`)
- **Build system**: Cargo workspace (resolver 2)
- **License**: MIT

### Key Dependencies

| Crate                           | Purpose                                                                              |
| ------------------------------- | ------------------------------------------------------------------------------------ |
| `serde` / `serde_json`          | JSON serialization and extractor deserialization                                     |
| `tokio` (full)                  | Async runtime for async dispatch and callback registries                             |
| `async-trait`                   | Async trait support for `CommandHandlerAsync`                                        |
| `anyhow`                        | Error handling (`Result<T>` = `anyhow::Result<T>`)                                   |
| `inventory`                     | Link-time collection for `#[command]` auto-registration (optional, `macros` feature) |
| `syn` / `quote` / `proc-macro2` | Proc-macro crate dependencies                                                        |

### Feature Flags

| Feature  | Default | Description                                                       |
| -------- | ------- | ----------------------------------------------------------------- |
| `macros` | Off     | Enables `#[command]` macro, `reg_all_commands()`, and `inventory` |
| `full`   | Off     | Enables all optional features                                     |

### Common Commands

```bash
# Build
cargo build

# Build with macros feature
cargo build --features macros

# Check (used by rust-analyzer / flycheck)
cargo check --workspace

# Test (if tests exist)
cargo test --workspace

# Test with all features
cargo test --workspace --all-features
```

### CI / CD

- **CI** (`.github/workflows/ci.yml`): Runs `check`, `build`, and `test` on every push/PR to `main` using nightly toolchain.
- **Publish** (`.github/workflows/publish.yml`): Triggered by pushing a `v*` tag. Publishes `cmdreg-macros` first, then `cmdreg` to crates.io.

#### Release Steps

1. Update `version` in the root `Cargo.toml` (both sub-crates inherit it via `workspace.package.version`).
2. Commit and push to `main`.
3. Tag and push:
   ```bash
   git tag v0.x.x
   git push origin v0.x.x
   ```
4. GitHub Actions will automatically build and publish both crates to crates.io.

> **Prerequisite**: The GitHub repository must have a secret named `CARGO_REGISTRY_TOKEN` containing a crates.io API token with publish permissions.

## Architecture & Design Patterns

### Three Dispatch Modes

1. **Sync** (`dispatch_sync.rs`) — `LazyLock<Arc<RwLock<CommandMap>>>` using `std::sync::RwLock`
2. **Async** (`dispatch_async.rs`) — `LazyLock<Arc<tokio::sync::RwLock<AsyncMap>>>` using `tokio::sync::RwLock`
3. **Callback** (`callback.rs`) — `LazyLock<Arc<tokio::sync::RwLock<CallbackMap>>>` for dynamic runtime handlers

### Handler Trait System

- `CommandHandler<T>` (sync) and `CommandHandlerAsync<T>` (async) are auto-implemented for functions with 0–10 extractor parameters via macro-generated impls.
- Handlers must be `Send + Sync + 'static`.
- Parameters are extracted from `CommandContext` using the `FromCommandArgs` trait.

### Extractor Pattern (Axum-Style)

- `Json<T>` wraps `serde::Deserialize` types and extracts from `CommandContext`.
- `CommandContext` is an enum: `String(&String)`, `Value(&serde_json::Value)`, or `None`.
- Extraction happens automatically before the handler is called.

### Macro Registration Flow

1. `#[command("prefix")]` or `#[command]` generates a registration function `__cmdreg_auto_reg_{fn_name}()`.
2. `inventory::submit!` registers it as a `CommandRegistration`.
3. `reg_all_commands()` iterates all collected registrations at startup.
4. With a prefix: command name is `"{prefix}.{function_name}"` (e.g., `"fs.read"`).
5. Without a prefix: command name is `"{function_name}"` (e.g., `"ping"`).

### Two `#[command]` Styles

#### Classic style (extractor-based)

- Handler uses `Json<T>` extractor patterns (e.g., `Json(args): Json<MyArgs>`) and returns `CommandResult`.
- The macro registers the original function directly — no code generation beyond the registration glue.

#### Plain style (auto-generated)

- Handler uses plain parameters (e.g., `path: String, recursive: bool`) and any `Serialize` return type.
- The macro auto-generates:
  - A hidden `#[derive(Deserialize)]` struct with `#[serde(rename_all = "camelCase")]` containing all parameters.
  - A hidden wrapper function that accepts `Json<GeneratedStruct>`, destructures fields, calls the original function, and wraps the return value.
- Return type mapping:
  - `T: Serialize` → `CommandResponse::json(value)`
  - `Result<T: Serialize>` → `CommandResponse::json(value?)`
  - `CommandResult` → passed through directly
  - `()` / no return → `Ok(CommandResponse::None)`
- Reference types (e.g., `&str`) in parameters are rejected at compile time with a clear error message.
- Style detection: if all parameters are simple `name: Type` patterns (not destructor patterns like `Json(x): Json<T>`), the plain style is used.

## Coding Conventions

- **Error handling**: Use `anyhow::Result<T>` everywhere. Propagate with `?`.
- **Naming**: Sync functions have no suffix; async functions use `_async` suffix; callback functions use `_callback` suffix.
- **Types**: PascalCase for public types (`CommandContext`, `CommandResponse`), snake_case for functions.
- **Sync command keys**: `&'static str` (compile-time strings). Callback keys: `String` (dynamic).
- **Thread safety**: All global registries use `Arc<RwLock<...>>`. All handlers require `Send + Sync + 'static`.
- **Unsafe code**: Minimal — only `unsafe impl Send for CommandContext<'_>` in `context.rs`.

## Important Notes for AI Agents

- **Nightly Rust is required.** The project uses `#![feature(closure_lifetime_binder)]` in `dispatch_async.rs` and `handler_async.rs`.
- **`tokio` runtime dependency**: Async and callback dispatch modules use `tokio::sync::RwLock`. Some sync registration functions internally create a new `tokio::runtime::Runtime` to block on async operations.
- **Max 10 handler parameters**: The macro-generated trait impls support functions with up to 10 extractor parameters.
- **No test suite is present** in the repository. When adding new features, consider adding tests.
- **The `cmdreg-macros` crate** is a proc-macro crate and cannot export non-macro items. It depends on `syn 2`, `quote`, and `proc-macro2`.
- **`lib.rs` is the public API surface.** All user-facing types and functions are re-exported from `lib.rs`. When adding new public items, ensure they are re-exported there.
- **Command key format**: `"{namespace}.{command_name}"` (e.g., `"fs.read"`) when using `#[command("prefix")]`, or just `"{command_name}"` (e.g., `"ping"`) when using `#[command]` without a prefix.
