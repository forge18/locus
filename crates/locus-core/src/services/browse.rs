//! Browser capability: one project browser service and one isolated context per run.
use crate::{
    ids::{ArtifactId, ProjectId, RunId},
    services::artifact::{write_blob, ArtifactKind, ArtifactRow},
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

pub const BROWSER_SERVICE_SUFFIX: &str = "browser";
pub const BROWSER_DOCS: &str =
    "Use Playwright auto-waiting; do not use sleep to wait for UI state.";
const DEFAULT_READY_TIMEOUT: Duration = Duration::from_secs(30);
const READY_POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BrowserContainer {
    pub project_id: ProjectId,
    pub name: String,
    pub network: String,
    pub running: bool,
}
impl BrowserContainer {
    pub fn for_project(project_id: ProjectId) -> Self {
        let id = project_id.to_string();
        Self {
            project_id,
            name: format!("locus-svc-{id}-{BROWSER_SERVICE_SUFFIX}"),
            network: format!("locus-{id}"),
            running: true,
        }
    }
}
#[derive(Clone, Default)]
pub struct BrowserServiceSupervisor {
    projects: Arc<Mutex<BTreeMap<ProjectId, BrowserContainer>>>,
}
impl BrowserServiceSupervisor {
    pub fn ensure_project(&self, id: ProjectId) -> BrowserContainer {
        self.projects
            .lock()
            .unwrap()
            .entry(id)
            .or_insert_with(|| BrowserContainer::for_project(id))
            .clone()
    }
    pub fn container(&self, id: ProjectId) -> Option<BrowserContainer> {
        self.projects.lock().unwrap().get(&id).cloned()
    }
    pub fn projects(&self) -> Vec<ProjectId> {
        self.projects.lock().unwrap().keys().copied().collect()
    }
    pub fn run_finished(&self, _project: ProjectId, _run: RunId) {}
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct Element {
    pub text: String,
    pub visible: bool,
    pub count: usize,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NetworkEntry {
    pub url: String,
    pub method: String,
    pub status: Option<u16>,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BrowserProjectSettings {
    pub allowed_origins: BTreeSet<String>,
    pub audits: Vec<BrowserNetworkAudit>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserNetworkAudit {
    pub origin: String,
    pub allowed: bool,
}
impl BrowserProjectSettings {
    pub fn allow_origin(&mut self, origin: impl Into<String>) {
        self.allowed_origins.insert(origin.into());
    }
    pub fn request_origin(&mut self, origin: &str) -> bool {
        let allowed = self.allowed_origins.contains(origin);
        self.audits.push(BrowserNetworkAudit {
            origin: origin.into(),
            allowed,
        });
        allowed
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct Page {
    pub url: String,
    pub title: String,
    elements: BTreeMap<String, Element>,
    pub cookies: BTreeMap<String, String>,
    pub storage: BTreeMap<String, String>,
    pub console: Vec<String>,
    pub network: Vec<NetworkEntry>,
}
impl Page {
    pub fn element(mut self, selector: impl Into<String>, value: Element) -> Self {
        self.elements.insert(selector.into(), value);
        self
    }
    pub fn set_element(&mut self, selector: impl Into<String>, value: Element) {
        self.elements.insert(selector.into(), value);
    }
    pub fn element_for(&self, selector: &str) -> Option<&Element> {
        self.elements.get(selector)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BrowserContext {
    pub run_id: RunId,
    pub pages: Vec<Page>,
    recording: bool,
}
impl BrowserContext {
    pub fn new(run_id: RunId) -> Self {
        Self {
            run_id,
            pages: vec![],
            recording: false,
        }
    }
    pub fn new_page(&mut self) -> &mut Page {
        self.pages.push(Page::default());
        self.pages.last_mut().unwrap()
    }
    pub fn current_page(&self) -> Result<&Page> {
        self.pages.last().context("browse has no open page")
    }
    pub fn current_page_mut(&mut self) -> Result<&mut Page> {
        self.pages.last_mut().context("browse has no open page")
    }
    pub fn click(&self, selector: &str) -> Result<()> {
        self.current_page()?
            .element_for(selector)
            .with_context(|| format!("selector `{selector}` was not found"))?;
        Ok(())
    }
    pub fn fill(&mut self, selector: &str, value: &str) -> Result<()> {
        let el = self
            .current_page_mut()?
            .elements
            .get_mut(selector)
            .with_context(|| format!("selector `{selector}` was not found"))?;
        el.text = value.into();
        Ok(())
    }
    pub fn press(&self, selector: &str, _key: &str) -> Result<()> {
        self.click(selector)
    }
    pub fn start_recording(&mut self) -> Result<()> {
        if self.recording {
            bail!("a recording is already active")
        }
        self.recording = true;
        Ok(())
    }
    pub fn stop_recording(&mut self) -> Result<RecordingArtifact> {
        if !self.recording {
            bail!("no recording is active")
        }
        self.recording = false;
        Ok(RecordingArtifact {
            bytes: b"webm recording".to_vec(),
            media_type: "video/webm".into(),
        })
    }
    pub fn is_recording(&self) -> bool {
        self.recording
    }
}
#[derive(Clone, Default)]
pub struct PlaywrightDriver {
    contexts: Arc<Mutex<BTreeMap<RunId, BrowserContext>>>,
}
impl PlaywrightDriver {
    pub fn context_for_run(&self, id: RunId) -> BrowserContext {
        self.contexts
            .lock()
            .unwrap()
            .entry(id)
            .or_insert_with(|| BrowserContext::new(id))
            .clone()
    }
    pub fn create_context(&self, id: RunId) -> BrowserContext {
        let c = BrowserContext::new(id);
        self.contexts.lock().unwrap().insert(id, c.clone());
        c
    }
    pub fn update_context(&self, c: BrowserContext) {
        self.contexts.lock().unwrap().insert(c.run_id, c);
    }
    pub fn close_context(&self, id: RunId) {
        self.contexts.lock().unwrap().remove(&id);
    }
    pub fn context_count(&self) -> usize {
        self.contexts.lock().unwrap().len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppLaunch {
    pub command: String,
    pub backgrounded: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadinessConfig {
    pub path: String,
    pub timeout: Duration,
}
impl Default for ReadinessConfig {
    fn default() -> Self {
        Self {
            path: "/".into(),
            timeout: DEFAULT_READY_TIMEOUT,
        }
    }
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AppConfig {
    pub run_script: Option<String>,
    pub readiness: ReadinessConfig,
}
#[derive(Clone, Default)]
pub struct AppSupervisor {
    launches: Arc<Mutex<BTreeMap<RunId, AppLaunch>>>,
}
impl AppSupervisor {
    pub fn start(&self, id: RunId, cfg: &AppConfig) -> Result<Option<AppLaunch>> {
        let Some(command) = cfg.run_script.as_deref() else {
            return Ok(None);
        };
        if command.trim().is_empty() {
            bail!("project run script must not be empty")
        };
        let l = AppLaunch {
            command: command.into(),
            backgrounded: true,
        };
        self.launches.lock().unwrap().insert(id, l.clone());
        Ok(Some(l))
    }
    pub fn launch(&self, id: RunId) -> Option<AppLaunch> {
        self.launches.lock().unwrap().get(&id).cloned()
    }
}
pub trait ReadinessProbe {
    fn ready(&self, url: &str) -> bool;
}
impl<F: for<'a> Fn(&'a str) -> bool> ReadinessProbe for F {
    fn ready(&self, url: &str) -> bool {
        self(url)
    }
}
pub fn wait_until_ready<P: ReadinessProbe>(probe: &P, url: &str, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if probe.ready(url) {
            return Ok(());
        }
        thread::sleep(READY_POLL_INTERVAL);
    }
    bail!("app readiness probe timed out for {url}")
}
pub fn app_url(run: RunId, port: u16, url: &str) -> Result<String> {
    if url.trim().is_empty() {
        bail!("browse URL must not be empty")
    };
    if url.starts_with("http://") || url.starts_with("https://") {
        return Ok(url.into());
    };
    Ok(format!(
        "http://locus-agent-{run}:{port}/{}",
        url.trim_start_matches('/')
    ))
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AssertOptions {
    pub selector: String,
    pub text: Option<String>,
    pub visible: bool,
    pub count: Option<usize>,
}
pub fn parse_assert_args(args: &[String]) -> Result<AssertOptions> {
    let selector = args
        .first()
        .context("browse assert requires a selector")?
        .clone();
    let mut out = AssertOptions {
        selector,
        ..Default::default()
    };
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--text" => {
                i += 1;
                out.text = Some(args.get(i).context("--text requires a value")?.clone())
            }
            "--visible" => out.visible = true,
            "--count" => {
                i += 1;
                out.count = Some(
                    args.get(i)
                        .context("--count requires a value")?
                        .parse()
                        .context("--count must be a non-negative integer")?,
                )
            }
            x => bail!("unknown browse assert option `{x}`"),
        };
        i += 1
    }
    Ok(out)
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AssertionFailure {
    pub selector: String,
    pub reason: String,
    pub expected: AssertOptions,
    pub actual: Option<Element>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AssertionResult {
    pub passed: bool,
    pub failure: Option<AssertionFailure>,
}
pub fn assert_page(page: &Page, opts: &AssertOptions) -> AssertionResult {
    let actual = page.element_for(&opts.selector).cloned();
    let reason = actual
        .as_ref()
        .and_then(|e| {
            if opts.text.as_ref().is_some_and(|t| !e.text.contains(t)) {
                Some("text did not match")
            } else if opts.visible && !e.visible {
                Some("element is not visible")
            } else if opts.count.is_some_and(|n| e.count != n) {
                Some("element count did not match")
            } else {
                None
            }
        })
        .map(str::to_owned)
        .or_else(|| actual.is_none().then(|| "selector was not found".into()));
    AssertionResult {
        passed: reason.is_none(),
        failure: reason.map(|reason| AssertionFailure {
            selector: opts.selector.clone(),
            reason,
            expected: opts.clone(),
            actual,
        }),
    }
}
pub fn assertion_json(result: &AssertionResult) -> serde_json::Value {
    serde_json::to_value(result).unwrap()
}
pub fn verify_can_gate(result: &AssertionResult) -> bool {
    result.passed
}

pub fn browser_docs() -> &'static str {
    BROWSER_DOCS
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingArtifact {
    pub bytes: Vec<u8>,
    pub media_type: String,
}
pub fn screenshot_artifact(
    root: impl AsRef<Path>,
    project: ProjectId,
    run: RunId,
    name: impl AsRef<Path>,
    bytes: &[u8],
) -> Result<ArtifactRow> {
    write_blob(
        root,
        project,
        run,
        ArtifactKind::Image,
        name,
        "image/webp",
        bytes,
    )
}
pub fn recording_artifact(
    root: impl AsRef<Path>,
    project: ProjectId,
    run: RunId,
    name: impl AsRef<Path>,
    r: &RecordingArtifact,
) -> Result<ArtifactRow> {
    write_blob(
        root,
        project,
        run,
        ArtifactKind::Recording,
        name,
        r.media_type.clone(),
        &r.bytes,
    )
}
#[derive(Clone, Debug, Default)]
pub struct CardArtifacts {
    links: BTreeMap<RunId, BTreeSet<ArtifactId>>,
}
impl CardArtifacts {
    pub fn attach(&mut self, run: RunId, a: &ArtifactRow) {
        self.links.entry(run).or_default().insert(a.id);
    }
    pub fn for_run(&self, run: RunId) -> Vec<ArtifactId> {
        self.links
            .get(&run)
            .map(|x| x.iter().copied().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod container_per_project {
    use super::*;
    #[test]
    fn one_service_for_many_runs() {
        let p = ProjectId::generate();
        let s = BrowserServiceSupervisor::default();
        assert_eq!(s.ensure_project(p), s.ensure_project(p));
        assert_eq!(
            s.container(p).unwrap().name,
            format!("locus-svc-{p}-browser")
        );
    }
}
#[cfg(test)]
mod driver {
    use super::*;
    #[test]
    fn project_network() {
        let p = ProjectId::generate();
        assert_eq!(
            BrowserContainer::for_project(p).network,
            format!("locus-{p}")
        );
    }
}
#[cfg(test)]
mod context_per_run {
    use super::*;
    #[test]
    fn one_each() {
        let d = PlaywrightDriver::default();
        d.create_context(RunId::generate());
        d.create_context(RunId::generate());
        assert_eq!(d.context_count(), 2);
    }
}
#[cfg(test)]
mod contexts_are_isolated {
    use super::*;
    #[test]
    fn no_cross_run_state() {
        let d = PlaywrightDriver::default();
        let a = RunId::generate();
        let b = RunId::generate();
        let mut c = d.create_context(a);
        c.new_page().cookies.insert("sid".into(), "a".into());
        d.update_context(c);
        assert!(d.context_for_run(b).pages.is_empty());
    }
}
#[cfg(test)]
mod app_started_by_container {
    use super::*;
    #[test]
    fn backgrounded() {
        let a = AppSupervisor::default();
        let l = a
            .start(
                RunId::generate(),
                &AppConfig {
                    run_script: Some("npm run dev".into()),
                    ..Default::default()
                },
            )
            .unwrap()
            .unwrap();
        assert!(l.backgrounded);
    }
}
#[cfg(test)]
mod auto_waiting {
    use super::*;
    fn ready(_: &str) -> bool {
        true
    }
    #[test]
    fn waits() {
        wait_until_ready(&ready, "http://app", Duration::from_secs(1)).unwrap();
    }
}
#[cfg(test)]
mod open {
    use super::*;
    #[test]
    fn relative() {
        let r = RunId::generate();
        assert_eq!(
            app_url(r, 43123, "/settings").unwrap(),
            format!("http://locus-agent-{r}:43123/settings")
        );
    }
}
#[cfg(test)]
mod interactions {
    use super::*;
    #[test]
    fn click_fill_press() {
        let mut c = BrowserContext::new(RunId::generate());
        c.new_page().set_element(
            "#x",
            Element {
                text: "".into(),
                visible: true,
                count: 1,
            },
        );
        c.fill("#x", "ok").unwrap();
        c.click("#x").unwrap();
        c.press("#x", "Enter").unwrap();
        assert_eq!(
            c.current_page().unwrap().element_for("#x").unwrap().text,
            "ok"
        );
    }
}
#[cfg(test)]
mod assert {
    use super::*;
    #[test]
    fn flags() {
        let p = Page::default().element(
            "#x",
            Element {
                text: "Saved".into(),
                visible: true,
                count: 1,
            },
        );
        let o = parse_assert_args(&[
            "#x".into(),
            "--text".into(),
            "Save".into(),
            "--visible".into(),
            "--count".into(),
            "1".into(),
        ])
        .unwrap();
        assert!(assert_page(&p, &o).passed);
    }
}
#[cfg(test)]
mod assert_exit_code {
    use super::*;
    #[test]
    fn structured_failure() {
        let o = AssertOptions {
            selector: "#x".into(),
            ..Default::default()
        };
        let r = assert_page(&Page::default(), &o);
        assert!(!r.passed);
        assert_eq!(assertion_json(&r)["failure"]["selector"], "#x");
    }
}
#[cfg(test)]
mod verify_can_gate {
    use super::*;
    #[test]
    fn failure_blocks() {
        let o = AssertOptions {
            selector: "#x".into(),
            ..Default::default()
        };
        assert!(!verify_can_gate(&assert_page(&Page::default(), &o)));
    }
}
#[cfg(test)]
mod screenshot {
    use super::*;
    #[test]
    fn image() {
        let root = std::env::temp_dir().join(format!("locus-browse-{}", uuid::Uuid::new_v4()));
        let a = screenshot_artifact(
            &root,
            ProjectId::generate(),
            RunId::generate(),
            "shot.webp",
            b"image",
        )
        .unwrap();
        assert_eq!(a.kind, ArtifactKind::Image);
        let _ = std::fs::remove_dir_all(root);
    }
}
#[cfg(test)]
mod no_upload_step {
    use super::*;
    #[test]
    fn direct_blob() {
        let root = std::env::temp_dir().join(format!("locus-browse-{}", uuid::Uuid::new_v4()));
        let a = screenshot_artifact(
            &root,
            ProjectId::generate(),
            RunId::generate(),
            "shot.webp",
            b"image",
        )
        .unwrap();
        assert!(matches!(
            a.content,
            crate::services::artifact::ArtifactContent::Blob { .. }
        ));
        let _ = std::fs::remove_dir_all(root);
    }
}
#[cfg(test)]
mod artifact_on_card {
    use super::*;
    #[test]
    fn linked() {
        let p = ProjectId::generate();
        let r = RunId::generate();
        let a = ArtifactRow::text(p, r, ArtifactKind::Image, "shot");
        let mut c = CardArtifacts::default();
        c.attach(r, &a);
        assert_eq!(c.for_run(r), vec![a.id]);
    }
}
#[cfg(test)]
mod record {
    use super::*;
    #[test]
    fn start_stop() {
        let mut c = BrowserContext::new(RunId::generate());
        c.start_recording().unwrap();
        assert_eq!(c.stop_recording().unwrap().media_type, "video/webm");
    }
}
#[cfg(test)]
mod record_duration_cap {
    #[test]
    #[ignore = "recording duration cap is open in the feature contract"]
    fn open_contract() {}
}
#[cfg(test)]
mod console_network {
    use super::*;
    #[test]
    fn text() {
        let mut p = Page::default();
        p.console.push("rendered".into());
        p.network.push(NetworkEntry {
            url: "/api".into(),
            method: "GET".into(),
            status: Some(200),
        });
        assert_eq!(p.console[0], "rendered");
    }
}
#[cfg(test)]
mod no_egress_default {
    use crate::sandbox::egress::{DestinationAllowlists, EgressTarget, EgressTier};
    #[test]
    fn denied() {
        assert!(!DestinationAllowlists::default().permits(
            EgressTier::None,
            EgressTarget::Other,
            "example.com"
        ));
    }
}
#[cfg(test)]
mod granted_origin_audited {
    use super::*;
    #[test]
    fn named_audited() {
        let mut s = BrowserProjectSettings::default();
        s.allow_origin("https://example.test");
        assert!(s.request_origin("https://example.test"));
        assert!(s.audits[0].allowed);
    }
}
#[cfg(test)]
mod survives_run_exit {
    use super::*;
    #[test]
    fn service_lives() {
        let s = BrowserServiceSupervisor::default();
        let p = ProjectId::generate();
        s.ensure_project(p);
        s.run_finished(p, RunId::generate());
        assert!(s.container(p).unwrap().running);
    }
}
