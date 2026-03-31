//! Tests for task cancellation and async utilities.

use std::sync::atomic::Ordering;

use crate::{CancellationToken, TaskCancellation, check_token_cancel, is_token_cancelled};

#[test]
fn task_cancellation_workflow() {
    // Test the complete workflow of TaskCancellation
    let token = CancellationToken::new();
    let cancel = TaskCancellation::new(Some(token.clone()));

    // Initially not cancelled
    assert!(!cancel.is_interrupted());
    assert!(cancel.check().is_ok());
    assert!(cancel.token().is_some());

    // Cancel the token
    token.cancel();
    assert!(is_token_cancelled(Some(&token)));

    // Check returns error after cancellation
    let result = cancel.check();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cancelled"));

    // Interrupt flag can be set independently
    cancel.interrupt_flag().store(true, Ordering::Relaxed);
    assert!(cancel.is_interrupted());
}

#[test]
fn task_cancellation_clones_share_state() {
    let token = CancellationToken::new();
    let cancel1 = TaskCancellation::new(Some(token));
    let cancel2 = cancel1.clone();

    // Both share the same interrupt flag
    cancel1.interrupt_flag().store(true, Ordering::Relaxed);
    assert!(cancel2.is_interrupted());
    assert!(cancel1.is_interrupted());
}

#[test]
fn task_cancellation_with_none_token() {
    let cancel = TaskCancellation::new(None);
    assert!(!cancel.is_interrupted());
    assert!(cancel.check().is_ok());
    assert!(cancel.token().is_none());
    assert!(!is_token_cancelled(None));
    assert!(check_token_cancel(None).is_ok());
}
