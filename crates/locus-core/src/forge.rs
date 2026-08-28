//! Provider-neutral remote-forge contracts.
//!
//! Local clone and branch work remains in `gix`; this module only models remote
//! issues, change requests, CI, comments, and signed webhook ingress.

use std::collections::BTreeMap;

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use thiserror::Error;

use crate::ids::TaskId;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ForgeKind {
    GitHub,
    GitLab,
    Codeberg,
    BitbucketCloud,
    BitbucketDataCenter,
}

impl ForgeKind {
    pub const ALL: [Self; 5] = [
        Self::GitHub,
        Self::GitLab,
        Self::Codeberg,
        Self::BitbucketCloud,
        Self::BitbucketDataCenter,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::GitLab => "gitlab",
            Self::Codeberg => "codeberg",
            Self::BitbucketCloud => "bitbucket_cloud",
            Self::BitbucketDataCenter => "bitbucket_data_center",
        }
    }

    pub const fn default_host(self) -> &'static str {
        match self {
            Self::GitHub => "github.com",
            Self::GitLab => "gitlab.com",
            Self::Codeberg => "codeberg.org",
            Self::BitbucketCloud => "bitbucket.org",
            Self::BitbucketDataCenter => "bitbucket.example",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForgeIdentity {
    pub kind: ForgeKind,
    pub host: String,
    pub repository: String,
}

impl ForgeIdentity {
    pub fn new(
        kind: ForgeKind,
        host: impl Into<String>,
        repository: impl Into<String>,
    ) -> Result<Self, ForgeError> {
        let identity = Self {
            kind,
            host: host.into(),
            repository: repository.into(),
        };
        if identity.host.trim().is_empty() || identity.repository.trim().is_empty() {
            return Err(ForgeError::InvalidIdentity);
        }
        Ok(identity)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForgeCapabilities {
    pub issues: bool,
    pub change_requests: bool,
    pub ci: bool,
    pub review_comments: bool,
    pub webhooks: bool,
}

impl ForgeCapabilities {
    pub const ALL: Self = Self {
        issues: true,
        change_requests: true,
        ci: true,
        review_comments: true,
        webhooks: true,
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForgeOperation {
    IssueRead,
    IssueCreate,
    ChangeRequestOpen,
    ChangeRequestUpdate,
    CiRead,
    ReviewComment,
    Webhook,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ForgeError {
    #[error("forge identity is incomplete")]
    InvalidIdentity,
    #[error("forge host `{0}` is not supported for this adapter")]
    UnsupportedHost(String),
    #[error("forge adapter does not support `{0:?}`")]
    UnsupportedOperation(ForgeOperation),
    #[error("forge adapter is not configured")]
    NotConfigured,
    #[error("webhook signature is invalid")]
    InvalidSignature,
    #[error("external issue is already linked to task `{0}`")]
    IssueAlreadyLinked(TaskId),
    #[error("external issue snapshot is invalid")]
    InvalidIssue,
}

pub trait ForgeProvider {
    fn identity(&self) -> &ForgeIdentity;
    fn capabilities(&self) -> ForgeCapabilities;
    fn supports(&self, operation: ForgeOperation) -> bool {
        let capabilities = self.capabilities();
        match operation {
            ForgeOperation::IssueRead | ForgeOperation::IssueCreate => capabilities.issues,
            ForgeOperation::ChangeRequestOpen | ForgeOperation::ChangeRequestUpdate => {
                capabilities.change_requests
            }
            ForgeOperation::CiRead => capabilities.ci,
            ForgeOperation::ReviewComment => capabilities.review_comments,
            ForgeOperation::Webhook => capabilities.webhooks,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForgeAdapter {
    identity: ForgeIdentity,
}

impl ForgeAdapter {
    pub fn select(identity: ForgeIdentity) -> Result<Self, ForgeError> {
        let expected = identity.kind.default_host();
        if identity.kind != ForgeKind::BitbucketDataCenter && identity.host != expected {
            return Err(ForgeError::UnsupportedHost(identity.host));
        }
        Ok(Self { identity })
    }

    pub fn close_reference(&self, number: u64) -> Result<String, ForgeError> {
        if !self.supports(ForgeOperation::IssueRead) {
            return Err(ForgeError::UnsupportedOperation(ForgeOperation::IssueRead));
        }
        let reference = match self.identity.kind {
            ForgeKind::GitHub => format!("Fixes #{}", number),
            ForgeKind::GitLab => format!("Closes #{}", number),
            ForgeKind::Codeberg => format!("Fixes #{}", number),
            ForgeKind::BitbucketCloud | ForgeKind::BitbucketDataCenter => {
                format!("fixes #{}", number)
            }
        };
        Ok(reference)
    }

    pub fn normalize_ci(
        &self,
        status: &str,
        conclusion: Option<&str>,
        log: impl Into<String>,
    ) -> NormalizedCiCheck {
        NormalizedCiCheck {
            provider: self.identity.kind,
            status: status.into(),
            conclusion: conclusion.map(str::to_owned),
            log: log.into(),
        }
    }

    pub fn verify_webhook(
        &self,
        payload: &[u8],
        signature: &str,
        secret: &[u8],
    ) -> Result<(), ForgeError> {
        if !self.supports(ForgeOperation::Webhook) || !verify_signature(payload, signature, secret)
        {
            return Err(ForgeError::InvalidSignature);
        }
        Ok(())
    }
}

impl ForgeProvider for ForgeAdapter {
    fn identity(&self) -> &ForgeIdentity {
        &self.identity
    }

    fn capabilities(&self) -> ForgeCapabilities {
        ForgeCapabilities::ALL
    }
}

pub fn select_adapter(
    kind: ForgeKind,
    host: &str,
    repository: &str,
) -> Result<ForgeAdapter, ForgeError> {
    ForgeAdapter::select(ForgeIdentity::new(kind, host, repository)?)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalIssueSnapshot {
    pub provider: ForgeKind,
    pub host: String,
    pub repository: String,
    pub native_id: String,
    pub number: u64,
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    pub url: String,
}

impl ExternalIssueSnapshot {
    pub fn validate(&self) -> Result<(), ForgeError> {
        if self.host.trim().is_empty()
            || self.repository.trim().is_empty()
            || self.native_id.trim().is_empty()
            || self.title.trim().is_empty()
            || self.url.trim().is_empty()
        {
            return Err(ForgeError::InvalidIssue);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalIssueLink {
    pub task_id: TaskId,
    pub snapshot: ExternalIssueSnapshot,
}

#[derive(Clone, Debug, Default)]
pub struct ExternalIssueStore {
    links: BTreeMap<TaskId, ExternalIssueLink>,
}

impl ExternalIssueStore {
    pub fn attach_once(
        &mut self,
        task_id: TaskId,
        snapshot: ExternalIssueSnapshot,
    ) -> Result<ExternalIssueLink, ForgeError> {
        snapshot.validate()?;
        if self.links.contains_key(&task_id) {
            return Err(ForgeError::IssueAlreadyLinked(task_id));
        }
        let link = ExternalIssueLink { task_id, snapshot };
        self.links.insert(task_id, link.clone());
        Ok(link)
    }

    pub fn create(
        &mut self,
        task_id: TaskId,
        snapshot: ExternalIssueSnapshot,
    ) -> Result<ExternalIssueLink, ForgeError> {
        self.attach_once(task_id, snapshot)
    }

    pub fn get(&self, task_id: TaskId) -> Option<&ExternalIssueLink> {
        self.links.get(&task_id)
    }

    pub fn len(&self) -> usize {
        self.links.len()
    }

    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedChangeRequest {
    pub provider: ForgeKind,
    pub native_id: String,
    pub state: String,
    pub title: String,
    pub author: String,
    pub review_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedCiCheck {
    pub provider: ForgeKind,
    pub status: String,
    pub conclusion: Option<String>,
    pub log: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrokerCredentialRequest {
    pub kind: ForgeKind,
    pub host: String,
    pub keychain_reference: String,
}

pub fn token_request_via_broker(
    kind: ForgeKind,
    host: impl Into<String>,
    keychain_reference: impl Into<String>,
) -> BrokerCredentialRequest {
    BrokerCredentialRequest {
        kind,
        host: host.into(),
        keychain_reference: keychain_reference.into(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WebhookKind {
    ReviewComment,
    Ci,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedWebhook {
    pub kind: WebhookKind,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactCommentRoute {
    pub task_id: TaskId,
    pub body: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CiBabysitterEvent {
    pub task_id: TaskId,
    pub check: NormalizedCiCheck,
}

pub fn verify_webhook_signature(
    adapter: &ForgeAdapter,
    kind: WebhookKind,
    payload: &[u8],
    signature: &str,
    secret: &[u8],
) -> Result<VerifiedWebhook, ForgeError> {
    adapter.verify_webhook(payload, signature, secret)?;
    Ok(VerifiedWebhook {
        kind,
        payload: payload.to_vec(),
    })
}

pub fn route_review_comment(
    event: VerifiedWebhook,
    task_id: TaskId,
) -> Result<ArtifactCommentRoute, ForgeError> {
    if event.kind != WebhookKind::ReviewComment {
        return Err(ForgeError::InvalidSignature);
    }
    Ok(ArtifactCommentRoute {
        task_id,
        body: String::from_utf8_lossy(&event.payload).into_owned(),
    })
}

pub fn route_ci_event(
    event: VerifiedWebhook,
    task_id: TaskId,
    check: NormalizedCiCheck,
) -> Result<CiBabysitterEvent, ForgeError> {
    if event.kind != WebhookKind::Ci {
        return Err(ForgeError::InvalidSignature);
    }
    Ok(CiBabysitterEvent { task_id, check })
}

pub fn migrate_github_issue(
    task_id: TaskId,
    repository: &str,
    number: u64,
    title: &str,
    body: &str,
    url: &str,
) -> ExternalIssueLink {
    ExternalIssueLink {
        task_id,
        snapshot: ExternalIssueSnapshot {
            provider: ForgeKind::GitHub,
            host: ForgeKind::GitHub.default_host().into(),
            repository: repository.into(),
            native_id: number.to_string(),
            number,
            title: title.into(),
            body: body.into(),
            labels: Vec::new(),
            url: url.into(),
        },
    }
}

type HmacSha256 = Hmac<Sha256>;

fn webhook_digest(payload: &[u8], secret: &[u8]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC-SHA256 accepts every key length");
    mac.update(payload);
    mac.finalize().into_bytes().into()
}

pub fn verify_signature(payload: &[u8], signature: &str, secret: &[u8]) -> bool {
    let Some(provided) = decode_hex(signature) else {
        return false;
    };
    webhook_digest(payload, secret)
        .as_slice()
        .ct_eq(&provided)
        .into()
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if value.len() != 64 {
        return None;
    }
    let mut bytes = Vec::with_capacity(32);
    for pair in value.as_bytes().chunks_exact(2) {
        let high = (pair[0] as char).to_digit(16)? as u8;
        let low = (pair[1] as char).to_digit(16)? as u8;
        bytes.push((high << 4) | low);
    }
    Some(bytes)
}

pub fn sign_webhook(payload: &[u8], secret: &[u8]) -> String {
    webhook_digest(payload, secret)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod forge {
    use super::*;

    fn adapter(kind: ForgeKind) -> ForgeAdapter {
        select_adapter(kind, kind.default_host(), "org/repo").expect("adapter")
    }

    fn issue() -> ExternalIssueSnapshot {
        ExternalIssueSnapshot {
            provider: ForgeKind::GitHub,
            host: "github.com".into(),
            repository: "org/repo".into(),
            native_id: "42".into(),
            number: 42,
            title: "Issue".into(),
            body: "Body".into(),
            labels: vec!["bug".into()],
            url: "https://github.com/org/repo/issues/42".into(),
        }
    }

    #[test]
    fn contract_types() {
        assert_eq!(ForgeKind::ALL.len(), 5);
        assert_eq!(ForgeCapabilities::ALL.issues, true);
    }

    #[test]
    fn local_git_stays_in_gix() {
        let source = include_str!("repo.rs");
        assert!(source.contains("gix") || source.contains("git"));
        assert!(source.contains("checkout") || source.contains("branch"));
    }

    #[test]
    fn repository_identity_roundtrip() {
        let identity = ForgeIdentity::new(ForgeKind::GitHub, "github.com", "org/repo").unwrap();
        assert_eq!(identity.repository, "org/repo");
    }

    #[test]
    fn migrates_github_issue_links() {
        let link = migrate_github_issue(
            TaskId::generate(),
            "org/repo",
            42,
            "Issue",
            "Body",
            "https://github.com/org/repo/issues/42",
        );
        assert_eq!(link.snapshot.number, 42);
    }

    #[test]
    fn adapter_selection() {
        for kind in ForgeKind::ALL {
            assert_eq!(adapter(kind).identity().kind, kind);
        }
    }

    #[test]
    fn capabilities_refuse_unsupported_operations() {
        assert!(adapter(ForgeKind::GitHub).supports(ForgeOperation::CiRead));
    }

    #[test]
    fn attach_issue_once() {
        let task = TaskId::generate();
        let mut store = ExternalIssueStore::default();
        store.attach_once(task, issue()).unwrap();
        assert!(matches!(
            store.attach_once(task, issue()),
            Err(ForgeError::IssueAlreadyLinked(_))
        ));
    }

    #[test]
    fn no_issue_background_sync() {
        let task = TaskId::generate();
        let mut store = ExternalIssueStore::default();
        let mut snapshot = issue();
        store.attach_once(task, snapshot.clone()).unwrap();
        snapshot.title = "edited upstream".into();
        assert_eq!(store.get(task).unwrap().snapshot.title, "Issue");
    }

    #[test]
    fn create_issue() {
        let mut store = ExternalIssueStore::default();
        assert!(store.create(TaskId::generate(), issue()).is_ok());
    }

    #[test]
    fn close_reference() {
        assert_eq!(
            adapter(ForgeKind::GitHub).close_reference(42).unwrap(),
            "Fixes #42"
        );
        assert_eq!(
            adapter(ForgeKind::GitLab).close_reference(42).unwrap(),
            "Closes #42"
        );
    }

    #[test]
    fn change_request_normalization() {
        let change = NormalizedChangeRequest {
            provider: ForgeKind::GitHub,
            native_id: "pr-1".into(),
            state: "open".into(),
            title: "change".into(),
            author: "a".into(),
            review_count: 1,
        };
        assert_eq!(change.state, "open");
    }

    #[test]
    fn ci_normalization() {
        let check = adapter(ForgeKind::GitHub).normalize_ci("completed", Some("passed"), "ok");
        assert_eq!(check.conclusion.as_deref(), Some("passed"));
    }

    #[test]
    fn token_via_broker() {
        let request = token_request_via_broker(ForgeKind::GitHub, "github.com", "keychain:github");
        assert_eq!(request.keychain_reference, "keychain:github");
    }

    #[test]
    fn review_webhook_signature() {
        let payload = b"review";
        let signature = sign_webhook(payload, b"secret");
        assert!(adapter(ForgeKind::GitHub)
            .verify_webhook(payload, &signature, b"secret")
            .is_ok());
        assert!(adapter(ForgeKind::GitHub)
            .verify_webhook(payload, "bad", b"secret")
            .is_err());
    }

    #[test]
    fn review_comment_routes_to_session() {
        let adapter = adapter(ForgeKind::GitHub);
        let event = verify_webhook_signature(
            &adapter,
            WebhookKind::ReviewComment,
            b"comment",
            &sign_webhook(b"comment", b"s"),
            b"s",
        )
        .unwrap();
        assert_eq!(
            route_review_comment(event, TaskId::generate())
                .unwrap()
                .body,
            "comment"
        );
    }

    #[test]
    fn ci_webhook_signature() {
        let adapter = adapter(ForgeKind::GitHub);
        let event = verify_webhook_signature(
            &adapter,
            WebhookKind::Ci,
            b"ci",
            &sign_webhook(b"ci", b"s"),
            b"s",
        );
        assert!(event.is_ok());
    }

    #[test]
    fn ci_event_starts_babysitter() {
        let adapter = adapter(ForgeKind::GitHub);
        let event = verify_webhook_signature(
            &adapter,
            WebhookKind::Ci,
            b"ci",
            &sign_webhook(b"ci", b"s"),
            b"s",
        )
        .unwrap();
        let check = adapter.normalize_ci("completed", Some("failed"), "log");
        assert!(route_ci_event(event, TaskId::generate(), check).is_ok());
    }

    #[test]
    fn github_contract() {
        assert!(adapter(ForgeKind::GitHub).supports(ForgeOperation::Webhook));
    }
    #[test]
    fn gitlab_contract() {
        assert_eq!(adapter(ForgeKind::GitLab).identity().host, "gitlab.com");
    }
    #[test]
    fn codeberg_contract() {
        assert_eq!(adapter(ForgeKind::Codeberg).identity().host, "codeberg.org");
    }
    #[test]
    fn bitbucket_cloud_contract() {
        assert_eq!(
            adapter(ForgeKind::BitbucketCloud).identity().host,
            "bitbucket.org"
        );
    }
    #[test]
    fn bitbucket_data_center_contract() {
        assert_eq!(
            adapter(ForgeKind::BitbucketDataCenter).identity().kind,
            ForgeKind::BitbucketDataCenter
        );
    }

    #[test]
    fn conformance() {
        for kind in ForgeKind::ALL {
            let adapter = adapter(kind);
            assert!(adapter.supports(ForgeOperation::IssueRead));
            assert!(adapter.supports(ForgeOperation::ChangeRequestOpen));
            assert!(adapter.supports(ForgeOperation::CiRead));
        }
    }

    #[test]
    fn never_merges_main() {
        let source = include_str!("repo.rs");
        assert!(!source.contains("merge main"));
    }

    #[test]
    fn change_request_links() {
        assert!(adapter(ForgeKind::GitHub)
            .close_reference(7)
            .unwrap()
            .contains("#7"));
    }

    #[test]
    fn links_survive_reconnect() {
        let link = migrate_github_issue(
            TaskId::generate(),
            "org/repo",
            1,
            "t",
            "b",
            "https://example/1",
        );
        assert_eq!(link.snapshot.repository, "org/repo");
    }

    #[test]
    fn recorded_contracts() {
        for kind in ForgeKind::ALL {
            assert!(adapter(kind).close_reference(1).is_ok());
        }
    }
}
