//! Configuration system - YAML-based config loading.
//!
//! Mirrors the Perl Netdisco configuration system - loads config.yml
//! as defaults, then overlays deployment.yml for local overrides.

pub mod settings;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::info;

pub use settings::*;

/// Load configuration from YAML files.
///
/// Configuration is loaded in layers:
/// 1. Built-in defaults (config.yml)
/// 2. Environment-specific overrides (environments/deployment.yml)
/// 3. Environment variable overrides
pub fn load_config(config_dir: Option<&Path>) -> Result<NetdiscoConfig> {
    let home = config_dir
        .map(PathBuf::from)
        .or_else(|| std::env::var("NETDISCO_HOME").ok().map(PathBuf::from))
        .or_else(|| dirs::home_dir())
        .context("Cannot determine home directory")?;

    // Load default config
    let default_config_path = home.join("config.yml");
    let mut config = if default_config_path.exists() {
        let contents = std::fs::read_to_string(&default_config_path)
            .with_context(|| format!("Failed to read {}", default_config_path.display()))?;
        serde_yaml::from_str::<NetdiscoConfig>(&contents)
            .with_context(|| format!("Failed to parse {}", default_config_path.display()))?
    } else {
        NetdiscoConfig::default()
    };

    // Load deployment overrides
    let env_config_path = home.join("environments").join("deployment.yml");
    if env_config_path.exists() {
        let contents = std::fs::read_to_string(&env_config_path)
            .with_context(|| format!("Failed to read {}", env_config_path.display()))?;
        let overrides: serde_yaml::Value = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse {}", env_config_path.display()))?;
        config.apply_overrides(&overrides)?;
        info!("Loaded config overrides from {}", env_config_path.display());
    }

    // Apply environment variable overrides
    config.apply_env_overrides();

    Ok(config)
}
