use super::*;

#[test]
fn test_repo_sync_outcome_none_when_no_values() {
    let outcome = RepoSyncOutcome {
        head_commit: None,
        head_branch: None,
    };
    assert!(outcome.head_commit.is_none());
    assert!(outcome.head_branch.is_none());
}

#[test]
fn test_repo_sync_outcome_with_values() {
    let outcome = RepoSyncOutcome {
        head_commit: Some("abc123".to_string()),
        head_branch: Some("main".to_string()),
    };
    assert_eq!(outcome.head_commit, Some("abc123".to_string()));
    assert_eq!(outcome.head_branch, Some("main".to_string()));
}

#[test]
fn test_infer_progress_unit_bytes_by_name() {
    assert_eq!(
        infer_progress_unit("downloading bytes", UNKNOWN, ProgressUnit::Unknown),
        ProgressUnit::Bytes
    );
    assert_eq!(
        infer_progress_unit("byte count", UNKNOWN, ProgressUnit::Unknown),
        ProgressUnit::Bytes
    );
}

#[test]
fn test_infer_progress_unit_bytes_by_id() {
    assert_eq!(
        infer_progress_unit("", *b"CLCB", ProgressUnit::Unknown),
        ProgressUnit::Bytes
    );
}

#[test]
fn test_infer_progress_unit_items() {
    assert_eq!(
        infer_progress_unit("processing files", UNKNOWN, ProgressUnit::Unknown),
        ProgressUnit::Items
    );
    assert_eq!(
        infer_progress_unit("object count", UNKNOWN, ProgressUnit::Unknown),
        ProgressUnit::Items
    );
}

#[test]
fn test_infer_progress_unit_items_by_id() {
    assert_eq!(
        infer_progress_unit("", *b"CLCF", ProgressUnit::Unknown),
        ProgressUnit::Items
    );
}

#[test]
fn test_infer_progress_unit_unknown() {
    assert_eq!(
        infer_progress_unit("something else", UNKNOWN, ProgressUnit::Unknown),
        ProgressUnit::Unknown
    );
}

#[test]
fn test_infer_progress_unit_preserves_current() {
    assert_eq!(
        infer_progress_unit("test", UNKNOWN, ProgressUnit::Items),
        ProgressUnit::Items
    );
    assert_eq!(
        infer_progress_unit("test", UNKNOWN, ProgressUnit::Bytes),
        ProgressUnit::Bytes
    );
}

#[test]
fn test_gix_progress_state_label_with_name() {
    let state = GixProgressState {
        name: "test-operation".to_string(),
        id: UNKNOWN,
        max: None,
        unit: ProgressUnit::Unknown,
        last_emitted_step: None,
    };
    assert_eq!(state.label(), "test-operation");
}

#[test]
fn test_gix_progress_state_label_with_id() {
    let state = GixProgressState {
        name: "".to_string(),
        id: *b"TEST",
        max: None,
        unit: ProgressUnit::Unknown,
        last_emitted_step: None,
    };
    assert_eq!(state.label(), "TEST");
}

#[test]
fn test_gix_progress_state_label_with_null_id() {
    let state = GixProgressState {
        name: "".to_string(),
        id: UNKNOWN,
        max: None,
        unit: ProgressUnit::Unknown,
        last_emitted_step: None,
    };
    assert_eq!(state.label(), "git operation");
}

#[test]
fn test_gix_progress_state_label_with_empty_name_and_id() {
    let state = GixProgressState {
        name: "".to_string(),
        id: UNKNOWN,
        max: None,
        unit: ProgressUnit::Unknown,
        last_emitted_step: None,
    };
    assert_eq!(state.label(), "git operation");
}
