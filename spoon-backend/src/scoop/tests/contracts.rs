use std::collections::BTreeMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use crate::scoop::{
    BucketUpdateSummary, execute_package_action_streaming_with_context, plan_package_action,
    update_buckets_streaming_with_context, ScoopIntegrationPort, SupplementalShimSpec,
};
use crate::tests::{block_on, temp_dir};
use crate::{BackendContext, BackendEvent, Result, SystemPort};

struct TestPorts;

impl SystemPort for TestPorts {
    fn home_dir(&self) -> PathBuf {
        PathBuf::from(".")
    }

    fn ensure_user_path_entry(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn ensure_process_path_entry(&self, _path: &Path) {}

    fn remove_user_path_entry(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn remove_process_path_entry(&self, _path: &Path) {}
}

impl ScoopIntegrationPort for TestPorts {
    fn supplemental_shims(
        &self,
        _package_name: &str,
        _current_root: &Path,
    ) -> Vec<SupplementalShimSpec> {
        Vec::new()
    }

    fn apply_integrations<'a>(
        &'a self,
        _package_name: &'a str,
        _current_root: &'a Path,
        _persist_root: &'a Path,
        _emit: &'a mut dyn FnMut(BackendEvent),
    ) -> Pin<Box<dyn Future<Output = Result<BTreeMap<String, String>>> + 'a>> {
        Box::pin(async { Ok(BTreeMap::new()) })
    }
}

#[test]
fn scoop_action_contract_uses_context() {
    let root = temp_dir("scoop-context-contract");
    let context = BackendContext::new(root, None, false, "x64", "default", TestPorts);
    let plan = plan_package_action("doctor", "git", "git", Some(&context.root));

    let mut sink = |_event: BackendEvent| {};
    let err = block_on(execute_package_action_streaming_with_context(
        &context, &plan, None, &mut sink,
    ))
    .expect_err("unsupported action should still route through context");

    match err {
        crate::BackendError::Other(message) => {
            assert!(message.contains("unsupported Scoop package action"));
        }
        other => panic!("expected unsupported action error, got {other:?}"),
    }
}

#[test]
fn bucket_sync_uses_backend_git_contract() {
    let root = temp_dir("bucket-context-contract");
    std::fs::create_dir_all(&root).expect("temp root should be created");
    let context = BackendContext::new(root, None, false, "x64", "default", TestPorts);

    let (lines, summary): (Vec<String>, BucketUpdateSummary) =
        block_on(update_buckets_streaming_with_context(&context, &[], None))
            .expect("empty registry should still be handled through context");

    assert!(lines.is_empty());
    assert_eq!(summary.updated, 0);
    assert_eq!(summary.skipped, 0);
}
