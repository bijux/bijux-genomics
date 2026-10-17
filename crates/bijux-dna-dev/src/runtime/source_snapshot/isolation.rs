//! Immutable Git-governed input conditions for retained and materialized native sources.

use anyhow::{ensure, Result};
use serde_json::{json, Value};

use super::{git, TrackedSourceSnapshot};
use crate::application::ci::local_tests::SourceIdentity;
use crate::runtime::{local_tests::source_identity, workspace::Workspace};

pub(crate) struct IsolatedSourceIdentity {
    expected: SourceIdentity,
    normalized_patch_sha256: String,
}

impl IsolatedSourceIdentity {
    /// Admit once before execution; terminal checks never replace this expected identity.
    pub(crate) fn capture(workspace: &Workspace) -> Result<Self> {
        require_checkout_root(workspace)?;
        let expected = source_identity(workspace)?;
        let snapshot = TrackedSourceSnapshot::capture(workspace)?;
        ensure!(
            source_identity(workspace)? == expected && snapshot.commit() == expected.sha,
            "source changed while acquiring isolated source identity"
        );
        Ok(Self { expected, normalized_patch_sha256: snapshot.patch_sha256() })
    }

    pub(crate) fn require_snapshot(&self, snapshot: &TrackedSourceSnapshot) -> Result<()> {
        ensure!(
            snapshot.commit() == self.expected.sha
                && snapshot.patch_sha256() == self.normalized_patch_sha256,
            "isolated source identity differs from the admitted tracked snapshot"
        );
        Ok(())
    }

    pub(crate) fn descriptor(&self) -> Value {
        json!({
            "schema_version":"bijux.ci.isolated_source_condition.v1",
            "expected":self.expected,
            "normalized_patch_sha256":self.normalized_patch_sha256,
            "input_scope":"Git HEAD, HEAD tree and tracked binary diff; nonignored untracked inputs are forbidden",
            "excluded_inputs":"Git-ignored paths, Git control files and external filesystem/tool inputs are not sealed by this condition",
            "artifact_policy":"ignored artifacts and separately owned stage-mutator roots may change",
            "dirty_source_policy":"preserve existing source_identity diff semantics; differing Git diff configuration is conservatively refused"
        })
    }

    pub(crate) fn verify(&self, workspace: &Workspace) -> Result<Value> {
        require_checkout_root(workspace)?;
        let observed = source_identity(workspace)?;
        ensure!(observed == self.expected, "isolated native source SHA/tree/patch changed");
        self.require_snapshot(&TrackedSourceSnapshot::capture(workspace)?)?;
        ensure!(
            source_identity(workspace)? == self.expected,
            "isolated native source changed during postcondition capture"
        );
        Ok(json!({"workspace":workspace.root,"unchanged":true,"observed":observed}))
    }

    /// Preserve all terminal observations even when a failed worker changed its inputs.
    pub(crate) fn postcondition(&self, workspace: &Workspace) -> Value {
        match self.verify(workspace) {
            Ok(value) => value,
            Err(error) => {
                json!({"workspace":workspace.root,"unchanged":false,"error":format!("{error:#}")})
            }
        }
    }
}

fn require_checkout_root(workspace: &Workspace) -> Result<()> {
    let root = String::from_utf8(git(workspace, &["rev-parse", "--show-toplevel"])?)?;
    ensure!(
        std::path::Path::new(root.trim()).canonicalize()? == workspace.root.canonicalize()?,
        "isolated native source must own its Git checkout root"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    fn fixture() -> Result<(tempfile::TempDir, Workspace)> {
        let (owner, workspace) = super::super::tests::source_fixture()?;
        std::fs::write(workspace.path(".gitignore"), b"/artifacts/\n")?;
        git(&workspace, &["add", "--", ".gitignore"])?;
        git(&workspace, &["commit", "--quiet", "-m", "fixture: declare artifact ownership"])?;
        Ok((owner, workspace))
    }

    #[test]
    fn tracked_shard_change_fails_original_condition_without_rebinding() -> Result<()> {
        let (owner, source) = fixture()?;
        let expected = IsolatedSourceIdentity::capture(&source)?;
        let snapshot = TrackedSourceSnapshot::capture(&source)?;
        expected.require_snapshot(&snapshot)?;
        let shard = Workspace { root: owner.path().join("shard") };
        snapshot.materialize(&shard.root)?;
        expected.verify(&shard)?;
        std::fs::write(shard.path("modified.txt"), b"worker changed tracked source\n")?;
        let observation = expected.postcondition(&shard);
        assert_eq!(observation["unchanged"], false);
        assert!(observation["error"]
            .as_str()
            .context("changed source observation must retain its error")?
            .contains("SHA/tree/patch"));
        assert_ne!(source_identity(&shard)?, expected.expected);
        expected.verify(&source)?;
        Ok(())
    }

    #[test]
    fn untracked_source_is_refused_before_admission_and_after_execution() -> Result<()> {
        let (owner, source) = fixture()?;
        std::fs::write(source.path("new-source.rs"), b"uncaptured product input\n")?;
        assert!(IsolatedSourceIdentity::capture(&source).is_err());
        std::fs::remove_file(source.path("new-source.rs"))?;
        let expected = IsolatedSourceIdentity::capture(&source)?;
        let shard = Workspace { root: owner.path().join("shard") };
        TrackedSourceSnapshot::capture(&source)?.materialize(&shard.root)?;
        std::fs::write(shard.path("new-source.rs"), b"worker added product input\n")?;
        assert_eq!(expected.postcondition(&shard)["unchanged"], false);
        assert!(expected
            .verify(&shard)
            .err()
            .context("untracked source must be refused")?
            .to_string()
            .contains("untracked"));
        Ok(())
    }

    #[test]
    fn ignored_artifact_outputs_preserve_declared_source_condition() -> Result<()> {
        let (owner, source) = fixture()?;
        let expected = IsolatedSourceIdentity::capture(&source)?;
        let shard = Workspace { root: owner.path().join("shard") };
        TrackedSourceSnapshot::capture(&source)?.materialize(&shard.root)?;
        std::fs::write(shard.path("artifacts/receipt.json"), b"{\"worker\":true}\n")?;
        expected.verify(&shard)?;
        assert!(expected.descriptor()["excluded_inputs"]
            .as_str()
            .context("source condition must declare excluded inputs")?
            .contains("Git-ignored"));
        Ok(())
    }

    #[test]
    fn dirty_tracked_inputs_keep_original_identity_and_reject_later_delta() -> Result<()> {
        let (owner, source) = fixture()?;
        std::fs::write(source.path("modified.txt"), b"admitted dirty source\n")?;
        let expected = IsolatedSourceIdentity::capture(&source)?;
        assert_eq!(expected.expected, source_identity(&source)?);
        let snapshot = TrackedSourceSnapshot::capture(&source)?;
        expected.require_snapshot(&snapshot)?;
        let shard = Workspace { root: owner.path().join("shard") };
        snapshot.materialize(&shard.root)?;
        expected.verify(&shard)?;
        std::fs::write(source.path("modified.txt"), b"later dirty source\n")?;
        assert!(expected.require_snapshot(&TrackedSourceSnapshot::capture(&source)?).is_err());
        assert!(expected.verify(&source).is_err());
        expected.verify(&shard)?;
        Ok(())
    }

    #[test]
    fn nested_directory_cannot_inherit_an_outer_checkout_condition() -> Result<()> {
        let (_owner, source) = fixture()?;
        let nested = Workspace { root: source.path("artifacts") };
        std::fs::create_dir(&nested.root)?;
        assert!(IsolatedSourceIdentity::capture(&nested).is_err());
        Ok(())
    }

    #[test]
    fn unchanged_tree_cannot_hide_a_different_source_commit() -> Result<()> {
        let (_owner, source) = fixture()?;
        let expected = IsolatedSourceIdentity::capture(&source)?;
        git(
            &source,
            &["commit", "--quiet", "--allow-empty", "-m", "fixture: change source commit"],
        )?;
        assert_eq!(source_identity(&source)?.tree, expected.expected.tree);
        assert_ne!(source_identity(&source)?.sha, expected.expected.sha);
        assert_eq!(expected.postcondition(&source)["unchanged"], false);
        Ok(())
    }

    #[test]
    fn dirty_diff_configuration_is_refused_without_normalizing_existing_identity() -> Result<()> {
        let (owner, source) = fixture()?;
        git(&source, &["config", "diff.noprefix", "true"])?;
        std::fs::write(source.path("modified.txt"), b"admitted dirty source\n")?;
        let expected = IsolatedSourceIdentity::capture(&source)?;
        assert_eq!(expected.expected, source_identity(&source)?);
        let snapshot = TrackedSourceSnapshot::capture(&source)?;
        expected.require_snapshot(&snapshot)?;
        let shard = Workspace { root: owner.path().join("shard") };
        snapshot.materialize(&shard.root)?;
        assert_eq!(std::fs::read(shard.path("modified.txt"))?, b"admitted dirty source\n");
        assert!(expected.verify(&shard).is_err());
        expected.verify(&source)?;
        Ok(())
    }
}
