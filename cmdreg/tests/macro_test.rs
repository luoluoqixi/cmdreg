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
