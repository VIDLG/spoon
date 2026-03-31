//! Tests for event emission and progress tracking.

use crate::{
    BackendEvent, CommandStatus, EventSink, FinishEvent, ProgressEvent, ProgressState, ProgressUnit,
};

#[test]
fn finish_event_all_variants() {
    let event = FinishEvent::success(Some("Done".to_string()));
    assert_eq!(event.status, CommandStatus::Success);

    let event = FinishEvent::cancelled("User cancelled");
    assert_eq!(event.status, CommandStatus::Cancelled);

    let event = FinishEvent::failed("Something went wrong");
    assert_eq!(event.status, CommandStatus::Failed);

    let event = FinishEvent::blocked("Missing dependency");
    assert_eq!(event.status, CommandStatus::Blocked);
}

#[test]
fn progress_event_all_constructors() {
    // Bytes progress
    let event = ProgressEvent::bytes("download", "file.zip", 1024, Some(2048));
    assert_eq!(event.kind, "download");
    assert_eq!(event.unit, ProgressUnit::Bytes);

    // Items progress
    let event = ProgressEvent::items("extract", "files", 5, 10);
    assert_eq!(event.unit, ProgressUnit::Items);

    // Steps progress
    let event = ProgressEvent::steps("build", "compiling", 2, 4);
    assert_eq!(event.unit, ProgressUnit::Steps);

    // Activity (indeterminate) progress
    let event = ProgressEvent::activity("cache", "warming up");
    assert_eq!(event.unit, ProgressUnit::Unknown);
    assert_eq!(event.current, None);
    assert_eq!(event.total, None);
}

#[test]
fn progress_event_with_modifiers() {
    let event = ProgressEvent::bytes("download", "file.zip", 1024, Some(2048))
        .with_id("download-123")
        .with_state(ProgressState::Completed);

    assert_eq!(event.id, Some("download-123".to_string()));
    assert_eq!(event.state, ProgressState::Completed);
}

#[test]
fn command_status_properties() {
    assert!(CommandStatus::Success.is_success());
    assert!(!CommandStatus::Failed.is_success());

    assert_eq!(CommandStatus::Success.as_str(), "success");
    assert_eq!(CommandStatus::Cancelled.as_str(), "cancelled");
}

#[test]
fn event_sink_emits_to_callback() {
    let mut events = Vec::new();
    let mut emit = |event: BackendEvent| events.push(event);
    {
        let mut sink = EventSink::new(Some(&mut emit));
        assert!(sink.is_enabled());
        sink.send(BackendEvent::Finished(FinishEvent::success(Some(
            "test".to_string(),
        ))));
    }
    assert_eq!(events.len(), 1);
}

#[test]
fn event_sink_disabled_without_callback() {
    let mut sink = EventSink::new(None);
    assert!(!sink.is_enabled());
    sink.send(BackendEvent::Finished(FinishEvent::success(None)));
    // Should not panic
}
