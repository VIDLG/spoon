#[cfg(test)]
mod tests {
    use crate::*;

    // ── Layout tests ──

    #[test]
    fn runtime_layout_from_root() {
        let layout = RuntimeLayout::from_root(std::path::Path::new("C:/spoon"));
        assert_eq!(layout.root, std::path::PathBuf::from("C:/spoon"));
        assert_eq!(layout.shims, std::path::PathBuf::from("C:/spoon/shims"));
        assert_eq!(layout.scoop.root, std::path::PathBuf::from("C:/spoon/scoop"));
        assert_eq!(
            layout.scoop.cache_root,
            std::path::PathBuf::from("C:/spoon/scoop/cache")
        );
        assert_eq!(
            layout.scoop.buckets_root,
            std::path::PathBuf::from("C:/spoon/scoop/buckets")
        );
        assert_eq!(
            layout.scoop.apps_root,
            std::path::PathBuf::from("C:/spoon/scoop/apps")
        );
        assert_eq!(
            layout.scoop.persist_root,
            std::path::PathBuf::from("C:/spoon/scoop/persist")
        );
        assert_eq!(layout.msvc.root, std::path::PathBuf::from("C:/spoon/msvc"));
        assert_eq!(
            layout.msvc.managed.root,
            std::path::PathBuf::from("C:/spoon/msvc/managed")
        );
        assert_eq!(
            layout.msvc.official.root,
            std::path::PathBuf::from("C:/spoon/msvc/official")
        );
    }

    #[test]
    fn scoop_layout_bucket_root() {
        let layout = RuntimeLayout::from_root(std::path::Path::new("C:/spoon"));
        assert_eq!(
            layout.scoop.bucket_root("main"),
            std::path::PathBuf::from("C:/spoon/scoop/buckets/main")
        );
    }

    #[test]
    fn scoop_layout_package_paths() {
        let layout = RuntimeLayout::from_root(std::path::Path::new("C:/spoon"));
        assert_eq!(
            layout.scoop.package_app_root("git"),
            std::path::PathBuf::from("C:/spoon/scoop/apps/git")
        );
        assert_eq!(
            layout.scoop.package_version_root("git", "2.40.0"),
            std::path::PathBuf::from("C:/spoon/scoop/apps/git/2.40.0")
        );
        assert_eq!(
            layout.scoop.package_current_root("git"),
            std::path::PathBuf::from("C:/spoon/scoop/apps/git/current")
        );
        assert_eq!(
            layout.scoop.package_persist_root("git"),
            std::path::PathBuf::from("C:/spoon/scoop/persist/git")
        );
    }

    #[test]
    fn msvc_layout_paths() {
        let layout = RuntimeLayout::from_root(std::path::Path::new("C:/spoon"));
        assert_eq!(
            layout.msvc.managed.cache_root,
            std::path::PathBuf::from("C:/spoon/msvc/managed/cache")
        );
        assert_eq!(
            layout.msvc.managed.toolchain_root,
            std::path::PathBuf::from("C:/spoon/msvc/managed/toolchain")
        );
        assert_eq!(
            layout.msvc.managed.manifest_root,
            std::path::PathBuf::from("C:/spoon/msvc/managed/cache/manifest")
        );
        assert_eq!(
            layout.msvc.official.instance_root,
            std::path::PathBuf::from("C:/spoon/msvc/official/instance")
        );
    }

    // ── Proxy tests ──

    #[test]
    fn normalize_proxy_empty() {
        assert!(normalize_proxy_url("").unwrap().is_none());
        assert!(normalize_proxy_url("  ").unwrap().is_none());
    }

    #[test]
    fn normalize_proxy_adds_http_scheme() {
        assert_eq!(
            normalize_proxy_url("127.0.0.1:7890").unwrap(),
            Some("http://127.0.0.1:7890".to_string())
        );
    }

    #[test]
    fn normalize_proxy_preserves_scheme() {
        assert_eq!(
            normalize_proxy_url("http://127.0.0.1:7890").unwrap(),
            Some("http://127.0.0.1:7890".to_string())
        );
        assert_eq!(
            normalize_proxy_url("https://proxy.example.com:8080").unwrap(),
            Some("https://proxy.example.com:8080".to_string())
        );
    }

    #[test]
    fn normalize_proxy_strips_trailing_slash() {
        assert_eq!(
            normalize_proxy_url("http://127.0.0.1:7890/").unwrap(),
            Some("http://127.0.0.1:7890".to_string())
        );
    }

    // ── Event system tests ──

    #[tokio::test]
    async fn event_bus_send_recv() {
        let (sender, mut receiver) = event_bus(64);

        sender.send(SpoonEvent::Stage(StageEvent::started(
            LifecycleStage::Acquiring,
        )));
        sender.send(SpoonEvent::Finished(FinishEvent::success(Some(
            "done".to_string(),
        ))));

        let event1 = receiver.recv().await.unwrap();
        assert!(matches!(
            event1,
            SpoonEvent::Stage(StageEvent {
                stage: LifecycleStage::Acquiring,
                state: ProgressState::Running,
                ..
            })
        ));

        let event2 = receiver.recv().await.unwrap();
        assert!(matches!(
            event2,
            SpoonEvent::Finished(FinishEvent {
                status: CommandStatus::Success,
                ..
            })
        ));
    }

    #[tokio::test]
    async fn event_bus_multiple_receivers() {
        let (sender, mut receiver1) = event_bus(64);
        let mut receiver2 = sender.subscribe();

        sender.send(SpoonEvent::Notice(NoticeEvent::info("hello")));

        let event1 = receiver1.recv().await.unwrap();
        let event2 = receiver2.recv().await.unwrap();

        assert!(matches!(event1, SpoonEvent::Notice(_)));
        assert!(matches!(event2, SpoonEvent::Notice(_)));
    }

    #[tokio::test]
    async fn event_bus_send_without_receivers_ok() {
        let (sender, _) = event_bus(64);
        // Should not panic or error when no receivers exist
        sender.send(SpoonEvent::Progress(ProgressEvent::bytes(
            ProgressKind::Download,
            "test",
            100,
            Some(200),
        )));
    }

    // ── Cancellation tests ──

    #[test]
    fn cancellation_token_not_cancelled() {
        let token = CancellationToken::new();
        assert!(!is_token_cancelled(Some(&token)));
        assert!(check_token_cancel(Some(&token)).is_ok());
    }

    #[test]
    fn cancellation_token_cancelled() {
        let token = CancellationToken::new();
        token.cancel();
        assert!(is_token_cancelled(Some(&token)));
        assert!(check_token_cancel(Some(&token)).is_err());
    }

    #[test]
    fn cancellation_token_none() {
        assert!(!is_token_cancelled(None));
        assert!(check_token_cancel(None).is_ok());
    }

    // ── Error tests ──

    #[test]
    fn core_error_context_chain() {
        let inner = CoreError::Cancelled;
        let outer = inner.context("install failed");
        let message = outer.to_string();
        assert!(message.contains("install failed"));
        assert!(message.contains("Cancelled"));
    }

    #[test]
    fn core_error_fs() {
        let err = CoreError::fs(
            "read",
            std::path::Path::new("/tmp/test.txt"),
            std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        );
        let message = err.to_string();
        assert!(message.contains("read"));
        assert!(message.contains("/tmp/test.txt"));
        assert!(message.contains("not found"));
    }
}
