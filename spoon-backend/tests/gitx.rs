mod common;

use common::{block_on, temp_dir, test_proxy};
use spoon_backend::{BackendEvent, CancellationToken, clone_repo};

#[test]
fn clone_repo_respects_pre_cancelled_job() {
    let target = temp_dir("gitx-cancel");
    let cancel = CancellationToken::new();
    cancel.cancel();

    let err = block_on(clone_repo(
        "https://example.invalid/repo.git",
        &target,
        Some("main"),
        "",
        Some(&cancel),
        None,
    ))
    .unwrap_err();

    assert!(err.to_string().contains("Cancelled by user."));
    assert!(!target.exists());
    let _ = std::fs::remove_dir_all(target);
}

#[test]
fn clone_repo_cancelled_before_start_emits_no_backend_events() {
    let target = temp_dir("gitx-cancelled");
    let cancel = CancellationToken::new();
    cancel.cancel();
    let mut streamed = Vec::new();

    let err = block_on(clone_repo(
        "https://example.invalid/repo.git",
        &target,
        Some("main"),
        "",
        Some(&cancel),
        Some(&mut |chunk: BackendEvent| streamed.push(chunk)),
    ))
    .unwrap_err();

    assert!(err.to_string().contains("Cancelled by user."));
    assert!(streamed.is_empty(), "streamed: {:?}", streamed);
    let _ = std::fs::remove_dir_all(target);
}

#[test]
#[ignore = "requires external network access and proxy availability"]
fn clone_repo_emits_progress_events() {
    let target = temp_dir("gitx-progress");
    let mut progress_events = Vec::new();
    let proxy = test_proxy();

    let _ = block_on(clone_repo(
        "https://github.com/rust-lang/rustlings",
        &target,
        None,
        &proxy,
        None,
        Some(&mut |event| {
            if let BackendEvent::Progress(_) = &event {
                progress_events.push(event);
            }
        }),
    ));

    // Should have at least some progress events
    assert!(
        !progress_events.is_empty(),
        "Expected progress events to be emitted"
    );

    let _ = std::fs::remove_dir_all(target);
}

#[test]
#[ignore = "requires external network access and proxy availability"]
fn clone_repo_respects_proxy_format() {
    let target = temp_dir("gitx-proxy");
    let proxy = test_proxy();

    let result = block_on(clone_repo(
        "https://github.com/rust-lang/rustlings",
        &target,
        None,
        &proxy,
        None,
        None,
    ));

    assert!(result.is_ok(), "Proxy-configured clone should succeed");

    let _ = std::fs::remove_dir_all(target);
}

#[test]
#[ignore = "requires external network access and proxy availability"]
fn clone_repo_real_remote_returns_outcome_with_commit_hash() {
    let target = temp_dir("gitx-remote-outcome");
    let proxy = test_proxy();

    let outcome = block_on(clone_repo(
        "https://github.com/rust-lang/rustlings",
        &target,
        None,
        &proxy,
        None,
        None,
    ))
    .unwrap();

    assert!(outcome.head_commit.is_some());

    let _ = std::fs::remove_dir_all(target);
}

#[test]
#[ignore = "requires external network access and proxy availability"]
fn clone_repo_real_remote_returns_outcome_with_branch_name() {
    let target = temp_dir("gitx-remote-branch");
    let proxy = test_proxy();

    let outcome = block_on(clone_repo(
        "https://github.com/rust-lang/rustlings",
        &target,
        Some("main"),
        &proxy,
        None,
        None,
    ))
    .unwrap();

    assert!(outcome.head_branch.is_some());

    let _ = std::fs::remove_dir_all(target);
}
