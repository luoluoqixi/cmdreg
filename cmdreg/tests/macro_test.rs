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
