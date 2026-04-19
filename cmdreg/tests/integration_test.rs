#![feature(closure_lifetime_binder)]

use cmdreg::*;
use serde::{Deserialize, Serialize};

// ============================================================
// Sync dispatch tests
// ============================================================

#[test]
fn test_sync_no_args() {
    fn hello() -> CommandResult {
        CommandResponse::json("hello world")
    }

    reg_command("test.sync.hello", hello).unwrap();

    let result = invoke_command("test.sync.hello", CommandContext::None).unwrap();
    assert!(result.is_some());
    let json = result.into_option().unwrap();
    assert_eq!(json, r#""hello world""#);
}

#[test]
fn test_sync_with_json_string_arg() {
    fn greet(Json(name): Json<String>) -> CommandResult {
        CommandResponse::json(format!("hi, {}", name))
    }

    reg_command("test.sync.greet", greet).unwrap();

    let args = serde_json::to_string("Alice").unwrap();
    let result = invoke_command("test.sync.greet", CommandContext::String(&args)).unwrap();
    let json = result.into_option().unwrap();
    assert_eq!(json, r#""hi, Alice""#);
}

#[test]
fn test_sync_with_json_value_arg() {
    #[derive(Deserialize)]
    struct AddArgs {
        a: i32,
        b: i32,
    }

    fn add(Json(args): Json<AddArgs>) -> CommandResult {
        CommandResponse::json(args.a + args.b)
    }

    reg_command("test.sync.add", add).unwrap();

    let value = serde_json::json!({"a": 3, "b": 4});
    let result = invoke_command("test.sync.add", CommandContext::Value(&value)).unwrap();
    let json = result.into_option().unwrap();
    assert_eq!(json, "7");
}

#[test]
fn test_sync_invoke_not_found() {
    let result = invoke_command("test.sync.nonexistent", CommandContext::None);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("command not found"));
}

#[test]
fn test_sync_unreg_command() {
    fn dummy() -> CommandResult {
        CommandResponse::json(true)
    }

    reg_command("test.sync.unreg_target", dummy).unwrap();
    assert!(invoke_command("test.sync.unreg_target", CommandContext::None).is_ok());

    unreg_command("test.sync.unreg_target").unwrap();
    assert!(invoke_command("test.sync.unreg_target", CommandContext::None).is_err());
}

#[test]
fn test_sync_get_keys_and_len() {
    fn noop() -> CommandResult {
        Ok(CommandResponse::None)
    }

    reg_command("test.sync.keys_a", noop).unwrap();
    reg_command("test.sync.keys_b", noop).unwrap();

    let keys = get_command_keys().unwrap();
    assert!(keys.contains(&"test.sync.keys_a".to_string()));
    assert!(keys.contains(&"test.sync.keys_b".to_string()));

    let len = get_command_len().unwrap();
    assert!(len >= 2);
}

// ============================================================
// Async dispatch tests
// ============================================================

#[tokio::test]
async fn test_async_no_args() {
    async fn ping() -> CommandResult {
        CommandResponse::json("pong")
    }

    tokio::task::spawn_blocking(|| reg_command_async("test.async.ping", ping).unwrap())
        .await
        .unwrap();

    let result = invoke_command_async("test.async.ping", CommandContext::None)
        .await
        .unwrap();
    assert_eq!(result.into_option().unwrap(), r#""pong""#);
}

#[tokio::test]
async fn test_async_with_json_arg() {
    async fn double(Json(n): Json<i32>) -> CommandResult {
        CommandResponse::json(n * 2)
    }

    tokio::task::spawn_blocking(|| reg_command_async("test.async.double", double).unwrap())
        .await
        .unwrap();

    let args = serde_json::to_string(&5).unwrap();
    let result = invoke_command_async("test.async.double", CommandContext::String(&args))
        .await
        .unwrap();
    assert_eq!(result.into_option().unwrap(), "10");
}

#[tokio::test]
async fn test_async_invoke_not_found() {
    let result = invoke_command_async("test.async.nonexistent", CommandContext::None).await;
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("async command not found"));
}

#[tokio::test]
async fn test_async_unreg_command() {
    async fn tmp() -> CommandResult {
        CommandResponse::json(true)
    }

    tokio::task::spawn_blocking(|| reg_command_async("test.async.unreg_target", tmp).unwrap())
        .await
        .unwrap();
    assert!(
        invoke_command_async("test.async.unreg_target", CommandContext::None)
            .await
            .is_ok()
    );

    tokio::task::spawn_blocking(|| unreg_command_async("test.async.unreg_target").unwrap())
        .await
        .unwrap();
    assert!(
        invoke_command_async("test.async.unreg_target", CommandContext::None)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn test_async_get_keys_and_len() {
    async fn noop() -> CommandResult {
        Ok(CommandResponse::None)
    }

    tokio::task::spawn_blocking(|| {
        reg_command_async("test.async.keys_a", noop).unwrap();
    })
    .await
    .unwrap();

    async fn noop2() -> CommandResult {
        Ok(CommandResponse::None)
    }

    tokio::task::spawn_blocking(|| {
        reg_command_async("test.async.keys_b", noop2).unwrap();
    })
    .await
    .unwrap();

    let keys = tokio::task::spawn_blocking(|| get_command_async_keys().unwrap())
        .await
        .unwrap();
    assert!(keys.contains(&"test.async.keys_a".to_string()));
    assert!(keys.contains(&"test.async.keys_b".to_string()));

    let len = tokio::task::spawn_blocking(|| get_command_async_len().unwrap())
        .await
        .unwrap();
    assert!(len >= 2);
}

// ============================================================
// Callback dispatch tests
// ============================================================

#[tokio::test]
async fn test_callback_basic() {
    tokio::task::spawn_blocking(|| {
        reg_command_callback(
            "test.cb.echo".to_string(),
            Box::new(|args| Box::pin(async move { Ok(args) })),
        )
        .unwrap();
    })
    .await
    .unwrap();

    let result = invoke_command_callback("test.cb.echo", Some("hello".to_string()))
        .await
        .unwrap();
    assert_eq!(result, Some("hello".to_string()));
}

#[tokio::test]
async fn test_callback_no_args() {
    tokio::task::spawn_blocking(|| {
        reg_command_callback(
            "test.cb.noargs".to_string(),
            Box::new(|_| Box::pin(async { Ok(Some("ok".to_string())) })),
        )
        .unwrap();
    })
    .await
    .unwrap();

    let result = invoke_command_callback("test.cb.noargs", None)
        .await
        .unwrap();
    assert_eq!(result, Some("ok".to_string()));
}

#[tokio::test]
async fn test_callback_lazy() {
    #[derive(Serialize, Deserialize)]
    struct Input {
        name: String,
    }

    tokio::task::spawn_blocking(|| {
        reg_command_callback(
            "test.cb.lazy".to_string(),
            Box::new(|args| {
                Box::pin(async move {
                    let input: Input = serde_json::from_str(&args.unwrap())?;
                    let greeting = format!("hello, {}", input.name);
                    Ok(Some(serde_json::to_string(&greeting)?))
                })
            }),
        )
        .unwrap();
    })
    .await
    .unwrap();

    let result: Option<String> =
        invoke_command_callback_lazy("test.cb.lazy", Some(Input { name: "Bob".into() }))
            .await
            .unwrap();
    assert_eq!(result, Some("hello, Bob".to_string()));
}

#[tokio::test]
async fn test_callback_not_found() {
    let result = invoke_command_callback("test.cb.nonexistent", None).await;
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("callback command not found"));
}

#[tokio::test]
async fn test_callback_unreg() {
    tokio::task::spawn_blocking(|| {
        reg_command_callback(
            "test.cb.unreg_target".to_string(),
            Box::new(|_| Box::pin(async { Ok(None) })),
        )
        .unwrap();
    })
    .await
    .unwrap();

    assert!(invoke_command_callback("test.cb.unreg_target", None)
        .await
        .is_ok());

    tokio::task::spawn_blocking(|| {
        unreg_command_callback(&"test.cb.unreg_target".to_string()).unwrap();
    })
    .await
    .unwrap();

    assert!(invoke_command_callback("test.cb.unreg_target", None)
        .await
        .is_err());
}

// ============================================================
// CommandContext tests
// ============================================================

#[test]
fn test_context_none() {
    let ctx = CommandContext::None;
    assert!(ctx.is_none());
    assert!(!ctx.is_some());
}

#[test]
fn test_context_from_string() {
    let s = "hello".to_string();
    let ctx = CommandContext::from_string(Some(&s));
    assert!(ctx.is_some());

    let ctx_none = CommandContext::from_string(None);
    assert!(ctx_none.is_none());
}

#[test]
fn test_context_from_value() {
    let v = serde_json::json!(42);
    let ctx = CommandContext::from_value(Some(&v));
    assert!(ctx.is_some());

    let ctx_none = CommandContext::from_value(None);
    assert!(ctx_none.is_none());
}

// ============================================================
// CommandResponse tests
// ============================================================

#[test]
fn test_response_json() {
    let resp = CommandResponse::json(42).unwrap();
    assert!(resp.is_some());
    assert!(!resp.is_none());
    assert_eq!(resp.into_option(), Some("42".to_string()));
}

#[test]
fn test_response_none() {
    let resp = CommandResponse::None;
    assert!(resp.is_none());
    assert!(!resp.is_some());
    assert_eq!(resp.into_option(), None);
}

#[test]
fn test_response_from_option() {
    let resp = CommandResponse::from_option(Some("data".to_string()));
    assert!(resp.is_some());
    assert_eq!(resp.into_option(), Some("data".to_string()));

    let resp = CommandResponse::from_option(None);
    assert!(resp.is_none());
}

// ============================================================
// Multi-parameter handler tests
// ============================================================

#[test]
fn test_sync_two_params() {
    fn concat(Json(a): Json<String>, Json(b): Json<String>) -> CommandResult {
        CommandResponse::json(format!("{}{}", a, b))
    }

    reg_command("test.sync.concat", concat).unwrap();

    let _args = serde_json::to_string(&["foo", "bar"]).unwrap();
    // Note: multi-param extractors all extract from the same context,
    // so both get the same JSON. This tests the trait impl compiles and runs.
    let value = serde_json::json!("hello");
    let result = invoke_command("test.sync.concat", CommandContext::Value(&value)).unwrap();
    assert!(result.is_some());
}

// ============================================================
// Handler returning None response
// ============================================================

#[test]
fn test_sync_handler_returns_none() {
    fn void_handler() -> CommandResult {
        Ok(CommandResponse::None)
    }

    reg_command("test.sync.void", void_handler).unwrap();

    let result = invoke_command("test.sync.void", CommandContext::None).unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_async_handler_returns_none() {
    async fn void_handler() -> CommandResult {
        Ok(CommandResponse::None)
    }

    tokio::task::spawn_blocking(|| reg_command_async("test.async.void", void_handler).unwrap())
        .await
        .unwrap();

    let result = invoke_command_async("test.async.void", CommandContext::None)
        .await
        .unwrap();
    assert!(result.is_none());
}
