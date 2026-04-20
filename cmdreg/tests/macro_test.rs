#![feature(closure_lifetime_binder)]
#![cfg(feature = "macros")]

use cmdreg::*;

// ============================================================
// #[command("prefix")] — sync with prefix
// ============================================================

#[command("test.macro")]
fn ping() -> CommandResult {
    CommandResponse::json("pong")
}

// ============================================================
// #[command("prefix")] — async with prefix
// ============================================================

#[command("test.macro")]
async fn async_hello() -> CommandResult {
    CommandResponse::json("hello async")
}

// ============================================================
// #[command] — sync without prefix
// ============================================================

#[command]
fn bare_cmd() -> CommandResult {
    CommandResponse::json("bare")
}

// ============================================================
// #[command] — async without prefix
// ============================================================

#[command]
async fn bare_async_cmd() -> CommandResult {
    CommandResponse::json("bare async")
}

// ============================================================
// #[command("prefix")] — with Json extractor
// ============================================================

#[command("test.macro")]
fn greet(Json(name): Json<String>) -> CommandResult {
    CommandResponse::json(format!("hi, {}", name))
}

#[command("test.macro")]
async fn async_greet(Json(name): Json<String>) -> CommandResult {
    CommandResponse::json(format!("hello, {}", name))
}

// ============================================================
// #[command("prefix")] — plain params (new style, sync)
// ============================================================

#[command("test.macro")]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// ============================================================
// #[command("prefix")] — single plain param (new style, sync)
// ============================================================

#[command("test.macro")]
fn echo(name: String) -> String {
    format!("echo: {}", name)
}

// ============================================================
// #[command("prefix")] — single plain param (new style, async)
// ============================================================

#[command("test.macro")]
async fn async_echo(name: String) -> String {
    format!("async echo: {}", name)
}

// ============================================================
// #[command("prefix")] — single struct param (new style, sync)
// ============================================================

#[derive(serde::Deserialize)]
struct Point {
    x: f64,
    y: f64,
}

#[command("test.macro")]
fn point_len(p: Point) -> f64 {
    (p.x * p.x + p.y * p.y).sqrt()
}

// ============================================================
// #[command("prefix")] — plain params (new style, async)
// ============================================================

#[command("test.macro")]
async fn async_multiply(x: i32, y: i32) -> i32 {
    x * y
}

// ============================================================
// #[command] — no params, plain return (new style)
// ============================================================

#[command]
fn get_version() -> String {
    "1.0.0".to_string()
}

#[command]
async fn async_get_version() -> String {
    "2.0.0".to_string()
}

// ============================================================
// #[command] — plain params, Result<T> return (new style)
// ============================================================

#[command("test.macro")]
fn safe_divide(a: f64, b: f64) -> anyhow::Result<f64> {
    if b == 0.0 {
        anyhow::bail!("division by zero");
    }
    Ok(a / b)
}

// ============================================================
// #[command] — no params, unit return (new style)
// ============================================================

#[command("test.macro")]
fn do_nothing() {}

// ============================================================
// Classic style with non-CommandResult returns (Json extractor + auto-wrap)
// ============================================================

#[command("test.classic")]
fn classic_bool(Json(name): Json<String>) -> bool {
    name == "yes"
}

#[command("test.classic")]
async fn async_classic_string(Json(n): Json<i32>) -> String {
    format!("value: {}", n)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Pair {
    left: String,
    right: String,
}

#[command("test.classic")]
fn classic_result(Json(pair): Json<Pair>) -> anyhow::Result<String> {
    if pair.left.is_empty() {
        anyhow::bail!("left is empty");
    }
    Ok(format!("{}-{}", pair.left, pair.right))
}

#[command("test.classic")]
fn classic_unit(Json(_n): Json<i32>) {}

// ============================================================
// Raw identifiers (r#type, r#move) as plain params
// ============================================================

#[command("test.macro")]
fn with_raw_idents(r#type: String, r#move: bool) -> String {
    format!("type={}, move={}", r#type, r#move)
}

#[command("test.macro")]
async fn async_with_raw_idents(r#type: String, r#override: bool) -> String {
    format!("type={}, override={}", r#type, r#override)
}

// ============================================================
// rename_all option
// ============================================================

#[command("test.macro", rename_all = "camelCase")]
fn with_camel_case(file_path: String, is_recursive: bool) -> String {
    format!("path={}, recursive={}", file_path, is_recursive)
}

#[command(rename_all = "SCREAMING_SNAKE_CASE")]
fn with_screaming_snake(my_value: i32) -> i32 {
    my_value * 2
}

// ============================================================
// Global rename_all from [package.metadata.cmdreg] in Cargo.toml
// No explicit rename_all — should use global "camelCase"
// ============================================================

#[command("test.global")]
fn global_rename_test(file_path: String, is_recursive: bool) -> String {
    format!("path={}, recursive={}", file_path, is_recursive)
}

#[command("test.global")]
async fn async_global_rename_test(max_depth: i32, include_hidden: bool) -> String {
    format!("depth={}, hidden={}", max_depth, include_hidden)
}

// ============================================================
// Tests
// ============================================================

#[test]
fn test_macro_reg_all_commands() {
    reg_all_commands().unwrap();

    // Verify prefixed sync command registered as "test.macro.ping"
    let keys = get_command_keys().unwrap();
    assert!(keys.contains(&"test.macro.ping".to_string()));

    // Verify prefixed sync command with args registered as "test.macro.greet"
    assert!(keys.contains(&"test.macro.greet".to_string()));

    // Verify bare sync command registered as "bare_cmd"
    assert!(keys.contains(&"bare_cmd".to_string()));
}

#[test]
fn test_macro_sync_invoke_prefixed() {
    reg_all_commands().unwrap();

    let result = invoke_command("test.macro.ping", CommandContext::None).unwrap();
    assert_eq!(result.into_option().unwrap(), r#""pong""#);
}

#[test]
fn test_macro_sync_invoke_bare() {
    reg_all_commands().unwrap();

    let result = invoke_command("bare_cmd", CommandContext::None).unwrap();
    assert_eq!(result.into_option().unwrap(), r#""bare""#);
}

#[test]
fn test_macro_sync_invoke_with_args() {
    reg_all_commands().unwrap();

    let args = serde_json::to_string("World").unwrap();
    let result = invoke_command("test.macro.greet", CommandContext::String(&args)).unwrap();
    assert_eq!(result.into_option().unwrap(), r#""hi, World""#);
}

#[tokio::test]
async fn test_macro_async_invoke_prefixed() {
    tokio::task::spawn_blocking(|| reg_all_commands().unwrap())
        .await
        .unwrap();

    let result = invoke_command_async("test.macro.async_hello", CommandContext::None)
        .await
        .unwrap();
    assert_eq!(result.into_option().unwrap(), r#""hello async""#);
}

#[tokio::test]
async fn test_macro_async_invoke_bare() {
    tokio::task::spawn_blocking(|| reg_all_commands().unwrap())
        .await
        .unwrap();

    let result = invoke_command_async("bare_async_cmd", CommandContext::None)
        .await
        .unwrap();
    assert_eq!(result.into_option().unwrap(), r#""bare async""#);
}

#[tokio::test]
async fn test_macro_async_invoke_with_args() {
    tokio::task::spawn_blocking(|| reg_all_commands().unwrap())
        .await
        .unwrap();

    let args = serde_json::to_string("Alice").unwrap();
    let result = invoke_command_async("test.macro.async_greet", CommandContext::String(&args))
        .await
        .unwrap();
    assert_eq!(result.into_option().unwrap(), r#""hello, Alice""#);
}

// ============================================================
// Tests for plain-param style (new style)
// ============================================================

#[test]
fn test_macro_plain_sync_add() {
    reg_all_commands().unwrap();

    let args = serde_json::json!({"a": 3, "b": 4});
    let result = invoke_command("test.macro.add", CommandContext::Value(&args)).unwrap();
    assert_eq!(result.into_option().unwrap(), "7");
}

#[tokio::test]
async fn test_macro_plain_async_multiply() {
    tokio::task::spawn_blocking(|| reg_all_commands().unwrap())
        .await
        .unwrap();

    let args = serde_json::json!({"x": 5, "y": 6});
    let result = invoke_command_async("test.macro.async_multiply", CommandContext::Value(&args))
        .await
        .unwrap();
    assert_eq!(result.into_option().unwrap(), "30");
}

#[test]
fn test_macro_plain_no_params_with_return() {
    reg_all_commands().unwrap();

    let result = invoke_command("get_version", CommandContext::None).unwrap();
    assert_eq!(result.into_option().unwrap(), r#""1.0.0""#);
}

#[tokio::test]
async fn test_macro_plain_async_no_params_with_return() {
    tokio::task::spawn_blocking(|| reg_all_commands().unwrap())
        .await
        .unwrap();

    let result = invoke_command_async("async_get_version", CommandContext::None)
        .await
        .unwrap();
    assert_eq!(result.into_option().unwrap(), r#""2.0.0""#);
}

#[test]
fn test_macro_plain_result_return() {
    reg_all_commands().unwrap();

    let args = serde_json::json!({"a": 10.0, "b": 2.0});
    let result = invoke_command("test.macro.safe_divide", CommandContext::Value(&args)).unwrap();
    assert_eq!(result.into_option().unwrap(), "5.0");

    // Test error case
    let args = serde_json::json!({"a": 10.0, "b": 0.0});
    let result = invoke_command("test.macro.safe_divide", CommandContext::Value(&args));
    assert!(result.is_err());
}

#[test]
fn test_macro_plain_unit_return() {
    reg_all_commands().unwrap();

    let result = invoke_command("test.macro.do_nothing", CommandContext::None).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_macro_plain_single_param_sync() {
    reg_all_commands().unwrap();

    // Single param: caller passes a struct with the param name as key
    let args = serde_json::json!({"name": "World"});
    let result = invoke_command("test.macro.echo", CommandContext::Value(&args)).unwrap();
    assert_eq!(result.into_option().unwrap(), r#""echo: World""#);
}

#[tokio::test]
async fn test_macro_plain_single_param_async() {
    tokio::task::spawn_blocking(|| reg_all_commands().unwrap())
        .await
        .unwrap();

    let args = serde_json::json!({"name": "Alice"});
    let result = invoke_command_async("test.macro.async_echo", CommandContext::Value(&args))
        .await
        .unwrap();
    assert_eq!(result.into_option().unwrap(), r#""async echo: Alice""#);
}

#[test]
fn test_macro_plain_single_struct_param() {
    reg_all_commands().unwrap();

    // Single struct param: caller wraps in {"p": ...}
    let args = serde_json::json!({"p": {"x": 3.0, "y": 4.0}});
    let result = invoke_command("test.macro.point_len", CommandContext::Value(&args)).unwrap();
    assert_eq!(result.into_option().unwrap(), "5.0");
}

// ============================================================
// Tests for classic style with non-CommandResult returns
// ============================================================

#[test]
fn test_classic_bool_return() {
    reg_all_commands().unwrap();

    let args = serde_json::to_string("yes").unwrap();
    let result =
        invoke_command("test.classic.classic_bool", CommandContext::String(&args)).unwrap();
    assert_eq!(result.into_option().unwrap(), "true");

    let args = serde_json::to_string("no").unwrap();
    let result =
        invoke_command("test.classic.classic_bool", CommandContext::String(&args)).unwrap();
    assert_eq!(result.into_option().unwrap(), "false");
}

#[tokio::test]
async fn test_classic_async_string_return() {
    tokio::task::spawn_blocking(|| reg_all_commands().unwrap())
        .await
        .unwrap();

    let args = serde_json::to_string(&42).unwrap();
    let result = invoke_command_async(
        "test.classic.async_classic_string",
        CommandContext::String(&args),
    )
    .await
    .unwrap();
    assert_eq!(result.into_option().unwrap(), r#""value: 42""#);
}

#[test]
fn test_classic_result_return() {
    reg_all_commands().unwrap();

    let args = serde_json::json!({"left": "a", "right": "b"});
    let result =
        invoke_command("test.classic.classic_result", CommandContext::Value(&args)).unwrap();
    assert_eq!(result.into_option().unwrap(), r#""a-b""#);

    // Error case
    let args = serde_json::json!({"left": "", "right": "b"});
    let result = invoke_command("test.classic.classic_result", CommandContext::Value(&args));
    assert!(result.is_err());
}

#[test]
fn test_classic_unit_return() {
    reg_all_commands().unwrap();

    let args = serde_json::to_string(&42).unwrap();
    let result =
        invoke_command("test.classic.classic_unit", CommandContext::String(&args)).unwrap();
    assert!(result.is_none());
}

// ============================================================
// Tests for raw identifiers (r#type, r#move)
// ============================================================

#[test]
fn test_macro_raw_idents_sync() {
    reg_all_commands().unwrap();

    // r#type → "type", r#move → "move" in JSON (no rename_all, serde strips r#)
    let args = serde_json::json!({"type": "file", "move": true});
    let result =
        invoke_command("test.macro.with_raw_idents", CommandContext::Value(&args)).unwrap();
    assert_eq!(result.into_option().unwrap(), r#""type=file, move=true""#);
}

#[tokio::test]
async fn test_macro_raw_idents_async() {
    tokio::task::spawn_blocking(|| reg_all_commands().unwrap())
        .await
        .unwrap();

    let args = serde_json::json!({"type": "dir", "override": false});
    let result = invoke_command_async(
        "test.macro.async_with_raw_idents",
        CommandContext::Value(&args),
    )
    .await
    .unwrap();
    assert_eq!(
        result.into_option().unwrap(),
        r#""type=dir, override=false""#
    );
}

// ============================================================
// Tests for rename_all option
// ============================================================

#[test]
fn test_macro_rename_all_camel_case() {
    reg_all_commands().unwrap();

    // With rename_all = "camelCase": file_path → "filePath", is_recursive → "isRecursive"
    let args = serde_json::json!({"filePath": "/tmp", "isRecursive": true});
    let result =
        invoke_command("test.macro.with_camel_case", CommandContext::Value(&args)).unwrap();
    assert_eq!(
        result.into_option().unwrap(),
        r#""path=/tmp, recursive=true""#
    );
}

#[test]
fn test_macro_rename_all_screaming_snake() {
    reg_all_commands().unwrap();

    // With rename_all = "SCREAMING_SNAKE_CASE": my_value → "MY_VALUE"
    let args = serde_json::json!({"MY_VALUE": 21});
    let result = invoke_command("with_screaming_snake", CommandContext::Value(&args)).unwrap();
    assert_eq!(result.into_option().unwrap(), "42");
}

#[test]
fn test_macro_no_rename_default() {
    reg_all_commands().unwrap();

    // Without rename_all: field names match Rust parameter names (snake_case)
    // add(a, b) uses plain names "a" and "b"
    let args = serde_json::json!({"a": 10, "b": 20});
    let result = invoke_command("test.macro.add", CommandContext::Value(&args)).unwrap();
    assert_eq!(result.into_option().unwrap(), "30");
}

// ============================================================
// Tests for global rename_all from Cargo.toml metadata
// ============================================================

#[test]
fn test_macro_global_rename_all() {
    reg_all_commands().unwrap();

    // No explicit rename_all on #[command] — uses global "camelCase" from
    // [package.metadata.cmdreg] in Cargo.toml.
    // file_path → "filePath", is_recursive → "isRecursive"
    let args = serde_json::json!({"filePath": "/tmp", "isRecursive": true});
    let result = invoke_command(
        "test.global.global_rename_test",
        CommandContext::Value(&args),
    )
    .unwrap();
    assert_eq!(
        result.into_option().unwrap(),
        r#""path=/tmp, recursive=true""#
    );
}

#[tokio::test]
async fn test_macro_global_rename_all_async() {
    tokio::task::spawn_blocking(|| reg_all_commands().unwrap())
        .await
        .unwrap();

    // max_depth → "maxDepth", include_hidden → "includeHidden"
    let args = serde_json::json!({"maxDepth": 5, "includeHidden": false});
    let result = invoke_command_async(
        "test.global.async_global_rename_test",
        CommandContext::Value(&args),
    )
    .await
    .unwrap();
    assert_eq!(result.into_option().unwrap(), r#""depth=5, hidden=false""#);
}

#[test]
fn test_macro_explicit_rename_overrides_global() {
    reg_all_commands().unwrap();

    // with_screaming_snake has explicit rename_all = "SCREAMING_SNAKE_CASE"
    // which overrides the global "camelCase".
    // my_value → "MY_VALUE" (not "myValue")
    let args = serde_json::json!({"MY_VALUE": 21});
    let result = invoke_command("with_screaming_snake", CommandContext::Value(&args)).unwrap();
    assert_eq!(result.into_option().unwrap(), "42");
}
