use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LockFileEntry {
    name: String,
    version: String,
    tarball_url: Option<String>,
    use_npm_fallback: bool,
    resolved_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockFile {
    version: String,
    packages: std::collections::HashMap<String, LockFileEntry>,
}

impl LockFile {
    pub(crate) fn new() -> Self {
        LockFile {
            version: env!("CARGO_PKG_VERSION").to_string(),
            packages: std::collections::HashMap::new(),
        }
    }

    pub(crate) fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let lockfile_path = Path::new("snpm.lockd");
        if !lockfile_path.exists() {
            return Ok(Self::new());
        }
        let content = fs::read_to_string(lockfile_path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub(crate) fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write("snpm.lockd", content)?;
        Ok(())
    }

    pub(crate) fn add_package(
        &mut self,
        name: String,
        version: String,
        tarball_url: Option<String>,
        use_npm_fallback: bool,
        resolved_version: String,
    ) {
        let key = format!("{}@{}", name, version);
        self.packages.insert(
            key,
            LockFileEntry {
                name,
                version,
                tarball_url,
                use_npm_fallback,
                resolved_version,
            },
        );
    }

    pub(crate) fn should_use_npm(&self, name: &str, version: &str) -> bool {
        let key = format!("{}@{}", name, version);
        self.packages
            .get(&key)
            .map_or(false, |entry| entry.use_npm_fallback)
    }
}
