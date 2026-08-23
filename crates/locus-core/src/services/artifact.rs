//! Reviewable run deliverables and their durable blob representation.

use crate::ids::{ArtifactId, CommentId, ProjectId, RunId};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const DEFAULT_COMPACTION_THRESHOLD: usize = 16 * 1024;
pub const ARTIFACT_ROOT: &str = "/var/lib/locus/artifacts";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    Plan,
    Diff,
    Diagram,
    Image,
    Recording,
    Walkthrough,
    Finding,
    Payload,
}

impl ArtifactKind {
    pub(crate) fn database_name(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Diff => "diff",
            Self::Diagram => "diagram",
            Self::Image => "image",
            Self::Recording => "recording",
            Self::Walkthrough => "walkthrough",
            Self::Finding => "finding",
            Self::Payload => "payload",
        }
    }

    pub(crate) fn from_database_name(name: &str) -> Result<Self> {
        match name {
            "plan" => Ok(Self::Plan),
            "diff" => Ok(Self::Diff),
            "diagram" => Ok(Self::Diagram),
            "image" => Ok(Self::Image),
            "recording" => Ok(Self::Recording),
            "walkthrough" => Ok(Self::Walkthrough),
            "finding" => Ok(Self::Finding),
            "payload" => Ok(Self::Payload),
            _ => bail!("unknown artifact kind `{name}`"),
        }
    }

    pub fn is_review(self) -> bool {
        !matches!(self, Self::Finding | Self::Payload)
    }
    pub fn is_media(self) -> bool {
        matches!(self, Self::Image | Self::Recording)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ArtifactContent {
    Text(String),
    Blob {
        path: PathBuf,
        media_type: String,
        sha256: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtifactRow {
    pub id: ArtifactId,
    pub project_id: ProjectId,
    pub run_id: RunId,
    pub kind: ArtifactKind,
    pub content: ArtifactContent,
    pub derived_cache: Option<serde_json::Value>,
    pub summary: Option<String>,
}

impl ArtifactRow {
    pub fn text(
        project_id: ProjectId,
        run_id: RunId,
        kind: ArtifactKind,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: ArtifactId::generate(),
            project_id,
            run_id,
            kind,
            content: ArtifactContent::Text(body.into()),
            derived_cache: None,
            summary: None,
        }
    }
}

pub fn blob_path(
    root: impl AsRef<Path>,
    project_id: ProjectId,
    run_id: RunId,
    name: impl AsRef<Path>,
) -> PathBuf {
    root.as_ref()
        .join(project_id.to_string())
        .join(run_id.to_string())
        .join(name)
}

fn validate_blob_name(name: &Path) -> Result<()> {
    if name.as_os_str().is_empty()
        || name.is_absolute()
        || name
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        bail!("artifact blob name must be one relative file name")
    }
    Ok(())
}

pub fn write_blob(
    root: impl AsRef<Path>,
    project_id: ProjectId,
    run_id: RunId,
    kind: ArtifactKind,
    name: impl AsRef<Path>,
    media_type: impl Into<String>,
    bytes: &[u8],
) -> Result<ArtifactRow> {
    let name = name.as_ref();
    validate_blob_name(name)?;
    let path = blob_path(root, project_id, run_id, name);
    let parent = path.parent().context("artifact blob has no parent")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("create artifact tree {}", parent.display()))?;
    fs::write(&path, bytes).with_context(|| format!("write artifact blob {}", path.display()))?;
    let sha256 = format!("{:x}", Sha256::digest(bytes));
    Ok(ArtifactRow {
        id: ArtifactId::generate(),
        project_id,
        run_id,
        kind,
        content: ArtifactContent::Blob {
            path,
            media_type: media_type.into(),
            sha256,
        },
        derived_cache: None,
        summary: None,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactComment {
    pub id: CommentId,
    pub artifact_id: ArtifactId,
    pub parent_id: Option<CommentId>,
    pub body: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommentDelivery {
    Live,
    Deferred,
}

#[derive(Default)]
pub struct ArtifactStore {
    rows: BTreeMap<ArtifactId, ArtifactRow>,
    comments: Vec<ArtifactComment>,
    live_sessions: Vec<RunId>,
}

impl ArtifactStore {
    pub fn put(&mut self, row: ArtifactRow) {
        self.rows.insert(row.id, row);
    }
    pub fn get(&self, id: ArtifactId) -> Option<&ArtifactRow> {
        self.rows.get(&id)
    }
    pub fn review_inbox(&self) -> Vec<&ArtifactRow> {
        self.rows
            .values()
            .filter(|row| row.kind.is_review())
            .collect()
    }
    pub fn comment(
        &mut self,
        artifact_id: ArtifactId,
        parent_id: Option<CommentId>,
        body: impl Into<String>,
    ) -> Result<CommentDelivery> {
        if !self.rows.contains_key(&artifact_id) {
            bail!("artifact {artifact_id} was not found")
        }
        let comment = ArtifactComment {
            id: CommentId::generate(),
            artifact_id,
            parent_id,
            body: body.into(),
        };
        self.comments.push(comment);
        let run_id = self.rows[&artifact_id].run_id;
        Ok(if self.live_sessions.contains(&run_id) {
            CommentDelivery::Live
        } else {
            CommentDelivery::Deferred
        })
    }
    pub fn comments(&self, artifact_id: ArtifactId) -> Vec<&ArtifactComment> {
        self.comments
            .iter()
            .filter(|comment| comment.artifact_id == artifact_id)
            .collect()
    }
    pub fn start_run(&mut self, run_id: RunId) -> Vec<&ArtifactComment> {
        self.live_sessions.push(run_id);
        self.comments
            .iter()
            .filter(|comment| self.rows[&comment.artifact_id].run_id == run_id)
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactionSettings {
    pub threshold: usize,
}
impl Default for CompactionSettings {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_COMPACTION_THRESHOLD,
        }
    }
}

pub fn compact_tool_result(
    store: &mut ArtifactStore,
    project_id: ProjectId,
    run_id: RunId,
    body: String,
    settings: &CompactionSettings,
) -> String {
    if body.len() <= settings.threshold {
        return body;
    }
    let mut row = ArtifactRow::text(project_id, run_id, ArtifactKind::Payload, body);
    row.summary = Some(format!("Tool result compacted; artifact {}", row.id));
    let summary = row.summary.clone().expect("summary set");
    store.put(row);
    summary
}

pub fn walkthrough(store: &ArtifactStore, project_id: ProjectId, run_id: RunId) -> ArtifactRow {
    let details = store
        .rows
        .values()
        .filter(|row| row.project_id == project_id && row.run_id == run_id)
        .map(|row| match &row.content {
            ArtifactContent::Text(text) => text.clone(),
            ArtifactContent::Blob { path, .. } => path.display().to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n");
    ArtifactRow::text(project_id, run_id, ArtifactKind::Walkthrough, details)
}

pub fn prune_media(rows: &mut Vec<(ArtifactRow, SystemTime, bool)>, now: SystemTime) {
    rows.retain(|(row, created, protected)| {
        !row.kind.is_media()
            || *protected
            || now.duration_since(*created).unwrap_or(Duration::ZERO)
                <= Duration::from_secs(30 * 24 * 60 * 60)
    });
}

pub fn read_artifact(row: &ArtifactRow) -> Result<Vec<u8>> {
    match &row.content {
        ArtifactContent::Text(text) => Ok(text.as_bytes().to_vec()),
        ArtifactContent::Blob { path, .. } => {
            fs::read(path).with_context(|| format!("read artifact blob {}", path.display()))
        }
    }
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod artifact {
    use super::*;
    use crate::ids::{ProjectId, RunId};
    use uuid::Uuid;

    fn ids() -> (ProjectId, RunId) {
        (ProjectId::generate(), RunId::generate())
    }
    fn root() -> PathBuf {
        std::env::temp_dir().join(format!("locus-artifact-{}", Uuid::new_v4()))
    }
    #[test]
    fn row() {
        let (project, run) = ids();
        let row = ArtifactRow::text(project, run, ArtifactKind::Plan, "plan");
        assert_eq!(row.kind, ArtifactKind::Plan);
        assert!(matches!(row.content, ArtifactContent::Text(_)));
    }
    #[test]
    fn kind_groups() {
        assert!(ArtifactKind::Plan.is_review());
        assert!(!ArtifactKind::Payload.is_review());
    }
    #[test]
    fn reference_never_in_inbox() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        store.put(ArtifactRow::text(
            project,
            run,
            ArtifactKind::Finding,
            "note",
        ));
        store.put(ArtifactRow::text(
            project,
            run,
            ArtifactKind::Plan,
            "review",
        ));
        assert_eq!(store.review_inbox().len(), 1);
    }
    #[test]
    fn text_is_a_row() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        let row = ArtifactRow::text(project, run, ArtifactKind::Diff, "diff");
        let id = row.id;
        store.put(row);
        assert_eq!(read_artifact(store.get(id).unwrap()).unwrap(), b"diff");
    }
    #[test]
    fn blob_tree() {
        let (project, run) = ids();
        assert_eq!(
            blob_path(ARTIFACT_ROOT, project, run, "a.png"),
            PathBuf::from(ARTIFACT_ROOT)
                .join(project.to_string())
                .join(run.to_string())
                .join("a.png")
        );
    }
    #[test]
    fn blob_name_cannot_escape_its_run_tree() {
        let (project, run) = ids();
        let root = root();
        let error = write_blob(
            &root,
            project,
            run,
            ArtifactKind::Image,
            "../../outside.png",
            "image/png",
            b"blob",
        )
        .expect_err("parent path must be refused");
        assert!(error.to_string().contains("relative file name"));
        assert!(!root.join("outside.png").exists());
    }

    #[test]
    fn sha256() {
        let (project, run) = ids();
        let root = root();
        let row = write_blob(
            &root,
            project,
            run,
            ArtifactKind::Image,
            "a.png",
            "image/png",
            b"blob",
        )
        .unwrap();
        assert!(
            matches!(row.content, ArtifactContent::Blob { ref sha256, .. } if sha256 == "fa2c8cc4f28176bbeed4b736df569a34c79cd3723e9ec42f9674b4d46ac6b8b8")
        );
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn comment_threads() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        let row = ArtifactRow::text(project, run, ArtifactKind::Plan, "p");
        let id = row.id;
        store.put(row);
        store.comment(id, None, "top").unwrap();
        let parent = store.comments(id)[0].id;
        store.comment(id, Some(parent), "reply").unwrap();
        assert_eq!(store.comments(id).len(), 2);
    }
    #[test]
    fn comment_steers_live() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        let row = ArtifactRow::text(project, run, ArtifactKind::Plan, "p");
        let id = row.id;
        store.put(row);
        store.start_run(run);
        assert_eq!(
            store.comment(id, None, "change").unwrap(),
            CommentDelivery::Live
        );
    }
    #[test]
    fn comment_deferred_delivery() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        let row = ArtifactRow::text(project, run, ArtifactKind::Plan, "p");
        let id = row.id;
        store.put(row);
        assert_eq!(
            store.comment(id, None, "later").unwrap(),
            CommentDelivery::Deferred
        );
        assert_eq!(store.start_run(run).len(), 1);
    }
    #[test]
    fn compacts_overflow() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        let summary = compact_tool_result(
            &mut store,
            project,
            run,
            "x".repeat(5),
            &CompactionSettings { threshold: 2 },
        );
        assert_eq!(store.rows.len(), 1);
        assert!(summary.contains("artifact"));
    }
    #[test]
    fn summary_with_handle() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        let summary = compact_tool_result(
            &mut store,
            project,
            run,
            "x".repeat(5),
            &CompactionSettings { threshold: 2 },
        );
        assert!(summary.starts_with("Tool result compacted; artifact "));
    }
    #[test]
    fn summary_ratio() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        let body = "x".repeat(100_000);
        let summary = compact_tool_result(
            &mut store,
            project,
            run,
            body.clone(),
            &CompactionSettings::default(),
        );
        assert!(summary.len() * 10 < body.len());
    }
    #[test]
    fn threshold_is_a_setting() {
        assert_eq!(
            CompactionSettings::default().threshold,
            DEFAULT_COMPACTION_THRESHOLD
        );
    }
    #[test]
    fn walkthrough_generates() {
        let (project, run) = ids();
        let mut store = ArtifactStore::default();
        store.put(ArtifactRow::text(
            project,
            run,
            ArtifactKind::Plan,
            "the plan",
        ));
        assert!(
            matches!(walkthrough(&store, project, run).content, ArtifactContent::Text(ref text) if text.contains("the plan"))
        );
    }
    #[test]
    fn media_retention() {
        let (project, run) = ids();
        let image = ArtifactRow::text(project, run, ArtifactKind::Image, "path");
        let keep = ArtifactRow::text(project, run, ArtifactKind::Recording, "path");
        let mut rows = vec![
            (image, SystemTime::UNIX_EPOCH, false),
            (keep, SystemTime::UNIX_EPOCH, true),
        ];
        prune_media(&mut rows, SystemTime::now());
        assert_eq!(rows.len(), 1);
    }
    #[test]
    fn text_never_pruned() {
        let (project, run) = ids();
        let mut rows = vec![(
            ArtifactRow::text(project, run, ArtifactKind::Plan, "plan"),
            SystemTime::UNIX_EPOCH,
            false,
        )];
        prune_media(&mut rows, SystemTime::now());
        assert_eq!(rows.len(), 1);
    }
    #[test]
    fn backup_covers_blobs() {
        let (project, run) = ids();
        let root = root();
        let row = write_blob(
            &root,
            project,
            run,
            ArtifactKind::Image,
            "image.png",
            "image/png",
            b"blob",
        )
        .unwrap();
        assert_eq!(read_artifact(&row).unwrap(), b"blob");
        let _ = fs::remove_dir_all(root);
    }
}
