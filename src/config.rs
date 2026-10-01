use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Global AIFlow user configuration (~/.config/aiflow/config.yaml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default = "default_workspace_roots")]
    pub workspace_roots: Vec<String>,

    #[serde(default = "default_preset")]
    pub default_preset: String,

    #[serde(default = "default_roles")]
    pub roles: HashMap<String, String>,
}

fn default_workspace_roots() -> Vec<String> {
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home);
        let mut roots = Vec::new();
        let gh = p.join("Shrinkhal-Github");
        if gh.exists() {
            roots.push(gh.to_string_lossy().to_string());
        }
        let proj = p.join("Projects");
        if proj.exists() {
            roots.push(proj.to_string_lossy().to_string());
        }
        if roots.is_empty() {
            roots.push(p.to_string_lossy().to_string());
        }
        roots
    } else {
        vec![".".to_string()]
    }
}

fn default_preset() -> String {
    "standard".to_string()
}

fn default_roles() -> HashMap<String, String> {
    let mut roles = HashMap::new();
    roles.insert("architect".to_string(), "claude-3-7-sonnet".to_string());
    roles.insert("implementer".to_string(), "codex".to_string());
    roles.insert("reviewer".to_string(), "gemini-2.5-pro".to_string());
    roles.insert("tester".to_string(), "gemini-2.5-pro".to_string());
    roles.insert("approver".to_string(), "human".to_string());
    roles
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            workspace_roots: default_workspace_roots(),
            default_preset: default_preset(),
            roles: default_roles(),
        }
    }
}

/// Returns path to ~/.config/aiflow/config.yaml
pub fn global_config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".config").join("aiflow").join("config.yaml"))
}

/// Load global configuration or create a default one if not found
pub fn load_or_init_global_config() -> GlobalConfig {
    if let Some(path) = global_config_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_yaml::from_str::<GlobalConfig>(&content) {
                    return config;
                }
            }
        } else {
            // Create parent directories and default file
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let default_cfg = GlobalConfig::default();
            if let Ok(yaml) = serde_yaml::to_string(&default_cfg) {
                let _ = fs::write(&path, yaml);
            }
            return default_cfg;
        }
    }
    GlobalConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_properties() {
        let cfg = GlobalConfig::default();
        assert_eq!(cfg.default_preset, "standard");
        assert!(cfg.roles.contains_key("architect"));
        assert!(cfg.roles.contains_key("implementer"));
        assert!(cfg.roles.contains_key("reviewer"));
    }

    #[test]
    fn test_serialize_config_yaml() {
        let cfg = GlobalConfig::default();
        let yaml = serde_yaml::to_string(&cfg).unwrap();
        assert!(yaml.contains("workspace_roots"));
        let parsed: GlobalConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.default_preset, "standard");
    }
}
