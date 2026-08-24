//! Marketplace manifest resolution and the per-agent tool allowlist.

use std::{
    collections::{BTreeMap, BTreeSet},
    io::Cursor,
};

use minisign::{PublicKey, PublicKeyBox, SignatureBox};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// A Minisign public key explicitly trusted by the local Locus settings.
#[derive(Default)]
pub struct TrustedKeyStore {
    keys: BTreeMap<String, PublicKey>,
}

impl TrustedKeyStore {
    pub fn from_public_keys(
        keys: impl IntoIterator<Item = String>,
    ) -> Result<Self, ToolAdmissionError> {
        let mut trusted = Self::default();
        for key in keys {
            trusted.add(key)?;
        }
        Ok(trusted)
    }

    pub fn add(&mut self, encoded_key: String) -> Result<(), ToolAdmissionError> {
        let key = PublicKeyBox::from_string(&encoded_key)
            .map_err(|_| ToolAdmissionError::InvalidTrustedKey)?
            .into_public_key()
            .map_err(|_| ToolAdmissionError::InvalidTrustedKey)?;
        self.keys.insert(encoded_key, key);
        Ok(())
    }

    fn verifies(&self, bytes: &[u8], signature: &SignatureBox) -> bool {
        self.keys.values().any(|key| {
            minisign::verify(key, signature, Cursor::new(bytes), true, false, false).is_ok()
        })
    }
}

/// The signed TOML document accompanying a custom tool binary.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolManifest {
    pub name: String,
    pub version: String,
    pub binary_sha256: String,
}

impl ToolManifest {
    pub fn new(name: impl Into<String>, version: impl Into<String>, binary: &[u8]) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            binary_sha256: binary_digest(binary),
        }
    }

    fn image_tool(&self) -> Result<ImageTool, ToolAdmissionError> {
        let tool = ImageTool::new(self.name.clone(), self.version.clone());
        if tool.name.trim().is_empty() || tool.version.trim().is_empty() {
            return Err(ToolAdmissionError::InvalidManifest);
        }
        if !is_sha256_digest(&self.binary_sha256) {
            return Err(ToolAdmissionError::InvalidManifest);
        }
        Ok(tool)
    }
}

/// Upload payloads remain separate so both the manifest and the executable are signed.
pub struct SignedToolUpload {
    pub manifest: Vec<u8>,
    pub manifest_signature: String,
    pub binary: Vec<u8>,
    pub binary_signature: String,
}

/// A resolved tool pin used to build an agent image.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ImageTool {
    pub name: String,
    pub version: String,
}

impl ImageTool {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

/// Aggregate enablement for a tool category, including the UI's mixed state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolGroupEnablement {
    Disabled,
    Mixed,
    Enabled,
}

impl ToolGroupEnablement {
    pub fn from_tools(enabled: impl IntoIterator<Item = bool>) -> Self {
        let mut any = false;
        let mut all = true;
        let mut seen = false;
        for value in enabled {
            seen = true;
            any |= value;
            all &= value;
        }
        if seen && all {
            Self::Enabled
        } else if any {
            Self::Mixed
        } else {
            Self::Disabled
        }
    }
}

/// Project-level tool removals from the enabled catalog baseline.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectToolScope {
    #[serde(default)]
    disabled_tools: BTreeSet<String>,
}

impl ProjectToolScope {
    pub fn new(disabled_tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            disabled_tools: disabled_tools.into_iter().map(Into::into).collect(),
        }
    }

    pub fn permits(&self, tool: &str) -> bool {
        !self.disabled_tools.contains(tool)
    }
    pub fn add(&mut self, tool: impl Into<String>) {
        self.disabled_tools.remove(&tool.into());
    }
    pub fn remove(&mut self, tool: impl Into<String>) {
        self.disabled_tools.insert(tool.into());
    }
    pub fn disabled_tools(&self) -> &BTreeSet<String> {
        &self.disabled_tools
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ImageRebuildLedger {
    last_scope: Option<ProjectToolScope>,
    rebuild_count: u32,
}

impl ImageRebuildLedger {
    pub fn apply_scope_change(&mut self, scope: ProjectToolScope) -> bool {
        if self.last_scope.as_ref() == Some(&scope) {
            return false;
        }
        self.last_scope = Some(scope);
        self.rebuild_count += 1;
        true
    }
    pub fn rebuild_count(&self) -> u32 {
        self.rebuild_count
    }
}

/// Workflow roles may further remove tools from the project's effective set.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoleToolScope {
    #[serde(default)]
    disabled_tools: BTreeSet<String>,
}

impl RoleToolScope {
    pub fn new(disabled_tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            disabled_tools: disabled_tools.into_iter().map(Into::into).collect(),
        }
    }

    pub fn permits(&self, tool: &str) -> bool {
        !self.disabled_tools.contains(tool)
    }
}

/// Built-ins and verified user tools, with image eligibility held separately from admission.
pub struct ToolCatalog {
    trusted_keys: TrustedKeyStore,
    admitted: BTreeSet<ImageTool>,
    enabled: BTreeSet<ImageTool>,
}

impl ToolCatalog {
    pub fn new(trusted_keys: TrustedKeyStore) -> Self {
        Self {
            trusted_keys,
            admitted: BTreeSet::new(),
            enabled: BTreeSet::new(),
        }
    }

    /// Add a Locus-shipped tool. Custom tools must use [`Self::admit_user_tool`] instead.
    pub fn add_builtin(&mut self, tool: ImageTool) -> Result<(), ToolAdmissionError> {
        self.admit(tool)
    }

    /// Verify both signed payloads and the manifest's binary digest before catalog admission.
    pub fn admit_user_tool(&mut self, upload: SignedToolUpload) -> Result<(), ToolAdmissionError> {
        verify_signature(
            &self.trusted_keys,
            &upload.manifest,
            &upload.manifest_signature,
            "manifest",
        )?;
        verify_signature(
            &self.trusted_keys,
            &upload.binary,
            &upload.binary_signature,
            "binary",
        )?;

        let manifest: ToolManifest = toml::from_str(
            std::str::from_utf8(&upload.manifest)
                .map_err(|_| ToolAdmissionError::InvalidManifest)?,
        )
        .map_err(|_| ToolAdmissionError::InvalidManifest)?;
        let tool = manifest.image_tool()?;
        if manifest.binary_sha256 != binary_digest(&upload.binary) {
            return Err(ToolAdmissionError::BinaryDigestMismatch);
        }

        self.admit(tool)
    }

    pub fn set_enabled(
        &mut self,
        name: impl AsRef<str>,
        version: impl AsRef<str>,
        enabled: bool,
    ) -> Result<(), ToolAdmissionError> {
        let tool = ImageTool::new(name.as_ref(), version.as_ref());
        if !self.admitted.contains(&tool) {
            return Err(ToolAdmissionError::UnknownTool);
        }
        if enabled {
            self.enabled.insert(tool);
        } else {
            self.enabled.remove(&tool);
        }
        Ok(())
    }

    /// Return a stable, sorted image set without any unadmitted or disabled tools.
    pub fn enabled_image_set(&self) -> Vec<ImageTool> {
        self.enabled.iter().cloned().collect()
    }

    /// Apply project then role subtraction to the enabled catalog baseline.
    pub fn scoped_image_set(
        &self,
        project: &ProjectToolScope,
        role: &RoleToolScope,
    ) -> Vec<ImageTool> {
        self.enabled
            .iter()
            .filter(|tool| project.permits(&tool.name) && role.permits(&tool.name))
            .cloned()
            .collect()
    }

    fn admit(&mut self, tool: ImageTool) -> Result<(), ToolAdmissionError> {
        if !self.admitted.insert(tool.clone()) {
            return Err(ToolAdmissionError::DuplicateTool);
        }
        self.enabled.insert(tool);
        Ok(())
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ToolAdmissionError {
    #[error("trusted Minisign public key is invalid")]
    InvalidTrustedKey,
    #[error("tool upload has no {0} Minisign signature")]
    MissingSignature(&'static str),
    #[error("tool upload has an invalid {0} Minisign signature")]
    InvalidSignature(&'static str),
    #[error("tool upload is not signed by a trusted Minisign key")]
    UntrustedSignature,
    #[error("tool manifest is invalid")]
    InvalidManifest,
    #[error("tool binary does not match the signed manifest digest")]
    BinaryDigestMismatch,
    #[error("tool is already admitted")]
    DuplicateTool,
    #[error("tool is not admitted to the catalog")]
    UnknownTool,
}

fn verify_signature(
    trusted_keys: &TrustedKeyStore,
    bytes: &[u8],
    encoded_signature: &str,
    part: &'static str,
) -> Result<(), ToolAdmissionError> {
    if encoded_signature.trim().is_empty() {
        return Err(ToolAdmissionError::MissingSignature(part));
    }
    let signature = SignatureBox::from_string(encoded_signature)
        .map_err(|_| ToolAdmissionError::InvalidSignature(part))?;
    if trusted_keys.verifies(bytes, &signature) {
        Ok(())
    } else {
        Err(ToolAdmissionError::UntrustedSignature)
    }
}

fn binary_digest(binary: &[u8]) -> String {
    format!("{:x}", Sha256::digest(binary))
}

fn is_sha256_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
use minisign::KeyPair;

#[cfg(test)]
fn signed_upload(key_pair: &KeyPair, manifest: ToolManifest, binary: Vec<u8>) -> SignedToolUpload {
    let manifest_bytes = toml::to_string(&manifest).unwrap().into_bytes();
    let sign = |bytes: &[u8]| {
        minisign::sign(None, &key_pair.sk, Cursor::new(bytes), None, None)
            .unwrap()
            .into_string()
    };

    SignedToolUpload {
        manifest: manifest_bytes.clone(),
        manifest_signature: sign(&manifest_bytes),
        binary_signature: sign(&binary),
        binary,
    }
}

#[test]
fn minisign_verification_rejects_unsigned_and_untrusted_uploads_before_admission() {
    let trusted = KeyPair::generate_unencrypted_keypair().unwrap();
    let untrusted = KeyPair::generate_unencrypted_keypair().unwrap();
    let trusted_key = trusted.pk.to_box().unwrap().into_string();
    let binary = b"#!/bin/sh\necho lint\n".to_vec();
    let manifest = ToolManifest::new("linty", "1.2.3", &binary);
    let mut catalog = ToolCatalog::new(TrustedKeyStore::from_public_keys([trusted_key]).unwrap());

    let untrusted_upload = signed_upload(&untrusted, manifest.clone(), binary.clone());
    assert_eq!(
        catalog.admit_user_tool(untrusted_upload).unwrap_err(),
        ToolAdmissionError::UntrustedSignature
    );
    assert!(catalog.enabled_image_set().is_empty());

    let mut unsigned_upload = signed_upload(&trusted, manifest.clone(), binary.clone());
    unsigned_upload.binary_signature.clear();
    assert_eq!(
        catalog.admit_user_tool(unsigned_upload).unwrap_err(),
        ToolAdmissionError::MissingSignature("binary")
    );
    assert!(catalog.enabled_image_set().is_empty());

    let mut tampered_upload = signed_upload(&trusted, manifest.clone(), binary.clone());
    tampered_upload.binary.push(b'!');
    assert_eq!(
        catalog.admit_user_tool(tampered_upload).unwrap_err(),
        ToolAdmissionError::UntrustedSignature
    );
    assert!(catalog.enabled_image_set().is_empty());

    catalog
        .admit_user_tool(signed_upload(&trusted, manifest, binary))
        .unwrap();
    assert_eq!(
        catalog.enabled_image_set(),
        vec![ImageTool::new("linty", "1.2.3")]
    );
}

#[test]
fn project_scope_add_remove() {
    let mut scope = ProjectToolScope::new(["rg"]);
    assert!(!scope.permits("rg"));
    scope.add("rg");
    assert!(scope.permits("rg"));
    scope.remove("sqlx");
    assert!(!scope.permits("sqlx"));
}

#[test]
fn image_rebuild_once_per_tool_change() {
    let mut ledger = ImageRebuildLedger::default();
    let scope = ProjectToolScope::default();
    assert!(ledger.apply_scope_change(scope.clone()));
    assert!(!ledger.apply_scope_change(scope));
    assert_eq!(ledger.rebuild_count(), 1);
}

#[test]
fn image_set_is_deterministic_and_only_contains_enabled_catalog_tools() {
    let mut catalog = ToolCatalog::new(TrustedKeyStore::default());
    catalog.add_builtin(ImageTool::new("zeta", "2.0")).unwrap();
    catalog.add_builtin(ImageTool::new("alpha", "1.0")).unwrap();
    catalog.set_enabled("zeta", "2.0", false).unwrap();

    assert_eq!(
        catalog.enabled_image_set(),
        vec![ImageTool::new("alpha", "1.0")]
    );
}
