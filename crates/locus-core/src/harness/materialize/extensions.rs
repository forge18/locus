//! The eight extension types, and the entries a run supplies for them.

use super::*;

pub const EXTENSIONS: [&str; 8] = [
    "agents",
    "commands",
    "hooks",
    "linters",
    "output-styles",
    "rules",
    "skills",
    "context",
];

/// One authored extension file. `frontmatter` and `body` are the plugin-facing representation;
/// `raw` preserves an authored file for the `dir` strategy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtensionEntry {
    pub name: String,
    #[serde(default)]
    pub frontmatter: Value,
    #[serde(default)]
    pub body: String,
    #[serde(skip)]
    pub raw: Option<String>,
}

impl ExtensionEntry {
    pub fn new(name: impl Into<String>, frontmatter: Value, body: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            frontmatter,
            body: body.into(),
            raw: None,
        }
    }

    pub fn with_raw(mut self, raw: impl Into<String>) -> Self {
        self.raw = Some(raw.into());
        self
    }

    pub(super) fn content(&self, strip_frontmatter: bool) -> String {
        if strip_frontmatter || self.frontmatter.is_null() || self.frontmatter == json!({}) {
            return self.body.clone();
        }
        if let Some(raw) = &self.raw {
            return raw.clone();
        }
        format!(
            "---\n{}\n---\n{}",
            serde_json::to_string(&self.frontmatter).expect("JSON frontmatter serializes"),
            self.body
        )
    }
}

/// Authored files grouped by the extension type that owns them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExtensionSet {
    entries: BTreeMap<String, Vec<ExtensionEntry>>,
}

/// Project toggles can remove extension groups or individual entries, but cannot add authored
/// extensions to a run.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectExtensionScope {
    #[serde(default)]
    disabled_extensions: BTreeSet<String>,
    #[serde(default)]
    disabled_entries: BTreeMap<String, BTreeSet<String>>,
}

impl ProjectExtensionScope {
    pub fn disable_extension(&mut self, extension: impl Into<String>) {
        self.disabled_extensions.insert(extension.into());
    }

    pub fn disable_entry(&mut self, extension: impl Into<String>, entry: impl Into<String>) {
        self.disabled_entries
            .entry(extension.into())
            .or_default()
            .insert(entry.into());
    }

    fn includes(&self, extension: &str, entry: &str) -> bool {
        !self.disabled_extensions.contains(extension)
            && !self
                .disabled_entries
                .get(extension)
                .is_some_and(|entries| entries.contains(entry))
    }
}

impl ExtensionSet {
    pub fn insert(&mut self, extension: impl Into<String>, entries: Vec<ExtensionEntry>) {
        self.entries.insert(extension.into(), entries);
    }

    /// Return the authored extensions after applying project-only subtraction.
    pub fn project_scoped(&self, scope: &ProjectExtensionScope) -> Self {
        Self {
            entries: self
                .entries
                .iter()
                .filter(|(extension, _)| !scope.disabled_extensions.contains(*extension))
                .map(|(extension, entries)| {
                    (
                        extension.clone(),
                        entries
                            .iter()
                            .filter(|entry| scope.includes(extension, &entry.name))
                            .cloned()
                            .collect(),
                    )
                })
                .collect(),
        }
    }

    pub fn entries(&self, extension: &str) -> &[ExtensionEntry] {
        self.entries
            .get(extension)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn sorted_entries(&self, extension: &str) -> Vec<&ExtensionEntry> {
        let mut entries = self.entries(extension).iter().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        entries
    }
}
