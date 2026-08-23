//! The fifth strategy: an executable that RETURNS the files to write, never writes them.

use super::*;
use crate::harness::materialize::extensions::ExtensionEntry;
use crate::harness::materialize::tree::GeneratedFile;

/// Plugin invocation settings. The plugin returns data; it never owns the config tree.
#[derive(Clone, Debug)]
pub struct PluginHost {
    pub program: PathBuf,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PluginRequest<'a> {
    jsonrpc: &'static str,
    id: u8,
    method: &'static str,
    params: PluginParams<'a>,
}

#[derive(Debug, Serialize)]
struct PluginParams<'a> {
    harness: &'a str,
    extension: &'a str,
    root: String,
    entries: Vec<&'a ExtensionEntry>,
    extensions: BTreeMap<&'a str, Vec<&'a ExtensionEntry>>,
}

#[derive(Debug, Deserialize)]
struct PluginResponse {
    #[serde(default)]
    result: Option<PluginResult>,
    #[serde(default)]
    files: Vec<PluginFile>,
}

#[derive(Debug, Deserialize)]
struct PluginResult {
    #[serde(default)]
    files: Vec<PluginFile>,
}

#[derive(Debug, Deserialize)]
struct PluginFile {
    path: PathBuf,
    #[serde(default = "default_mode")]
    mode: u32,
    content: String,
}

const fn default_mode() -> u32 {
    0o644
}

impl PluginHost {
    /// Invoke one plugin once for the complete plugin extension set of a run.
    pub(super) fn materialize(
        &self,
        harness: &str,
        root: &Path,
        extensions: BTreeMap<&str, Vec<&ExtensionEntry>>,
    ) -> Result<Vec<GeneratedFile>, MaterializeError> {
        let before = snapshot(root)?;
        let entries = extensions.values().flatten().copied().collect();
        let request = PluginRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "materialize",
            params: PluginParams {
                harness,
                extension: "all",
                root: root.display().to_string(),
                entries,
                extensions,
            },
        };
        let mut child = Command::new(&self.program)
            .args(&self.args)
            .env("ROOT", root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| MaterializeError::PluginFailed {
                program: self.program.clone(),
                message: source.to_string(),
            })?;
        child
            .stdin
            .take()
            .expect("piped stdin exists")
            .write_all(
                format!(
                    "{}\n",
                    serde_json::to_string(&request).expect("request serializes")
                )
                .as_bytes(),
            )
            .map_err(|source| MaterializeError::PluginFailed {
                program: self.program.clone(),
                message: source.to_string(),
            })?;
        let output = child
            .wait_with_output()
            .map_err(|source| MaterializeError::PluginFailed {
                program: self.program.clone(),
                message: source.to_string(),
            })?;
        if snapshot(root)? != before {
            restore_snapshot(root, &before)?;
            return Err(MaterializeError::PluginWroteDirectly);
        }
        if !output.status.success() {
            return Err(MaterializeError::PluginFailed {
                program: self.program.clone(),
                message: String::from_utf8_lossy(&output.stderr).trim().into(),
            });
        }
        let response: PluginResponse = serde_json::from_slice(&output.stdout)?;
        let files = response
            .result
            .map(|result| result.files)
            .unwrap_or(response.files);
        files
            .into_iter()
            .map(|file| {
                let path = path_under_root(root, &file.path)?;
                Ok(GeneratedFile {
                    path,
                    mode: file.mode,
                    content: file.content,
                })
            })
            .collect()
    }
}
