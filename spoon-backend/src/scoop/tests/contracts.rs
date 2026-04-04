use std::path::Path;

use async_trait::async_trait;
use crate::scoop::{
    AppliedIntegration, BucketUpdateSummary, ScoopIntegrationPort, SupplementalShimSpec,
    execute_package_action_streaming_with_context, plan_package_action, update_buckets_streaming_with_context,
};
use crate::tests::{block_on, temp_dir};
use crate::{BackendContext, BackendEvent, Result, SystemPort};

struct TestPorts;

impl SystemPort for TestPorts {
    fn ensure_user_path_entry(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn ensure_process_path_entry(&self, _path: &Path) {}

    fn remove_user_path_entry(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn remove_process_path_entry(&self, _path: &Path) {}
}

#[async_trait(?Send)]
impl ScoopIntegrationPort for TestPorts {
    fn supplemental_shims(
        &self,
        _package_name: &str,
        _current_root: &Path,
    ) -> Vec<SupplementalShimSpec> {
        Vec::new()
    }

    async fn apply_integrations(
        &self,
        _package_name: &str,
        _current_root: &Path,
        _persist_root: &Path,
        _emit: &mut dyn FnMut(BackendEvent),
    ) -> Result<Vec<AppliedIntegration>> {
        Ok(Vec::new())
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
        crate::BackendError::UnsupportedOperation { domain, operation } => {
            assert_eq!(domain, "Scoop package");
            assert_eq!(operation, "action");
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
