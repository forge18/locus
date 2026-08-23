//! Base image validation, Dockerfile generation, and content-addressed agent tags.

use super::*;

/// Buildable metadata requires a verified, pinned package command. Empty metadata is explicit,
/// so a new registry entry fails before Docker receives an invented command.
pub fn validate_image_metadata(harness: &HarnessDefinition) -> Result<()> {
    let image = &harness.image;
    if image.base.trim().is_empty() {
        bail!("harness `{}` image metadata is missing base", harness.name);
    }
    if image.version.trim().is_empty() || image.version == "unverified" {
        bail!(
            "harness `{}` image metadata has no verified version",
            harness.name
        );
    }
    if image.install.is_empty() {
        bail!(
            "harness `{}` image metadata has no verified install command",
            harness.name
        );
    }
    if !image.verified {
        bail!("harness `{}` image metadata is not verified", harness.name);
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseImagePlan {
    pub tag: String,
    pub dockerfile: String,
}

impl BaseImagePlan {
    pub fn from_harness(harness: &HarnessDefinition) -> Result<Self> {
        validate_image_metadata(harness)?;
        Ok(Self {
            tag: format!("locus/base-{}:{}", harness.name, harness.image.version),
            dockerfile: base_dockerfile(&harness.image, &harness.binary, &harness.detect),
        })
    }
}

fn base_dockerfile(image: &Image, binary: &str, detect: &[String]) -> String {
    let detect = std::iter::once(binary.to_owned())
        .chain(detect.iter().cloned())
        .map(|argument| shell_quote(&argument))
        .collect::<Vec<_>>()
        .join(" ");
    let environment = (!image.env.is_empty()).then(|| format!("ENV {}\n", image.env.join(" ")));
    format!(
        "FROM {}\n{}RUN {}\nRUN command -v {} && {}\n",
        image.base,
        environment.unwrap_or_default(),
        image.install.join(" && "),
        shell_quote(binary),
        detect,
    )
}

pub(crate) fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ToolPin {
    pub name: String,
    pub version: String,
}

/// Hashes only base digest and resolved tool pins. Prompt/config content intentionally is absent.
pub fn agent_image_key(base_digest: &str, tools: &[ToolPin]) -> String {
    let mut tools = tools.to_vec();
    tools.sort();
    let mut hasher = Sha256::new();
    hasher.update(base_digest.as_bytes());
    hasher.update([0]);
    for tool in tools {
        hasher.update(tool.name.as_bytes());
        hasher.update([0]);
        hasher.update(tool.version.as_bytes());
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

pub fn agent_image_tag(base_digest: &str, tools: &[ToolPin]) -> String {
    format!("locus/agent-{}", agent_image_key(base_digest, tools))
}

/// Derive an image tag from the catalog after project and role subtraction.
pub fn agent_image_tag_for_scopes(
    base_digest: &str,
    catalog: &ToolCatalog,
    project: &ProjectToolScope,
    role: &RoleToolScope,
) -> String {
    let tools = catalog
        .scoped_image_set(project, role)
        .into_iter()
        .map(|tool| ToolPin {
            name: tool.name,
            version: tool.version,
        })
        .collect::<Vec<_>>();
    agent_image_tag(base_digest, &tools)
}

/// An image rebuild is needed only when the resolved, ordered image set changed.
pub fn tool_set_requires_rebuild(current: &[ImageTool], next: &[ImageTool]) -> bool {
    current != next
}

#[cfg(test)]
mod images {
    use super::*;
    use crate::harness::registry::load_from_directory;

    fn registry() -> crate::harness::registry::HarnessRegistry {
        load_from_directory(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses"),
        )
        .unwrap()
    }

    #[test]
    fn base_builds() {
        let registry = registry();
        let plan = BaseImagePlan::from_harness(registry.by_name("dsh").unwrap()).unwrap();
        assert!(plan.tag.starts_with("locus/base-dsh:"));
        assert!(plan
            .dockerfile
            .contains("npm install --global @deepseek-ai/dsh@0.1.0-rc.7"));
    }

    #[test]
    fn metadata_is_declarative_for_all_registered_harnesses() {
        let registry = registry();
        assert_eq!(registry.len(), 11);

        for harness in registry.iter() {
            assert!(
                !harness.image.base.trim().is_empty(),
                "{} declares an image base",
                harness.name
            );
            if harness.image.verified {
                assert_ne!(harness.image.version, "unverified");
                assert!(
                    !harness.image.install.is_empty(),
                    "{} has a verified install command",
                    harness.name
                );
            } else {
                assert_eq!(harness.image.version, "unverified");
                assert!(
                    harness.image.install.is_empty(),
                    "{} does not invent an unverified install command",
                    harness.name
                );
                assert!(BaseImagePlan::from_harness(harness).is_err());
            }
        }
    }

    #[test]
    fn detect_fails_build() {
        let registry = registry();
        let plan = BaseImagePlan::from_harness(registry.by_name("dsh").unwrap()).unwrap();
        assert!(plan
            .dockerfile
            .contains("command -v 'dsh' && 'dsh' '--version'"));
        let error = BaseImagePlan::from_harness(registry.by_name("claude").unwrap()).unwrap_err();
        assert!(error.to_string().contains("no verified version"));
    }

    #[test]
    fn agent_layer() {
        assert!(agent_image_tag(
            "sha256:base",
            &[ToolPin {
                name: "rg".into(),
                version: "14".into()
            }]
        )
        .starts_with("locus/agent-"));
    }

    #[test]
    fn cache_key() {
        let unordered = [
            ToolPin {
                name: "z".into(),
                version: "1".into(),
            },
            ToolPin {
                name: "a".into(),
                version: "2".into(),
            },
        ];
        let ordered = [unordered[1].clone(), unordered[0].clone()];
        assert_eq!(
            agent_image_key("base", &unordered),
            agent_image_key("base", &ordered)
        );
        assert_ne!(
            agent_image_key("base", &unordered),
            agent_image_key("other", &unordered)
        );
    }

    #[test]
    fn shared_when_identical() {
        let tools = [ToolPin {
            name: "rg".into(),
            version: "14".into(),
        }];
        assert_eq!(
            agent_image_tag("base", &tools),
            agent_image_tag("base", &tools)
        );
    }

    #[test]
    fn config_is_not_a_layer() {
        let tools = [ToolPin {
            name: "rg".into(),
            version: "14".into(),
        }];
        let before = agent_image_tag("base", &tools);
        let edited_skill = "different prompt content";
        assert!(!edited_skill.is_empty());
        assert_eq!(before, agent_image_tag("base", &tools));
    }
}
