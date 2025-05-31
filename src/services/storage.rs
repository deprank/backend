use anyhow::Result;
use ghrepo::GHRepo;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tracing::{info, warn};
use uuid::Uuid;

use crate::errors::ApiError;

pub struct StorageService {
    cache_dir: PathBuf,
}

impl StorageService {
    pub fn new(cache_dir: &Path) -> Self {
        Self { cache_dir: cache_dir.to_path_buf() }
    }

    /// Download and store GitHub repository,
    /// and return the path of the cached directory.
    pub async fn fetch(&self, url: &str) -> Result<PathBuf> {
        // Extract repository name from URL
        let repo = GHRepo::from_url(url)?;

        // Create project directory to store the repository,
        // To avoid conflicts, create a subfolder with repository name + unique ID
        // @FIXME: using repo's latest commit hash as subfolder
        let dir = PathBuf::from(format!("{}/{}", repo, Uuid::new_v4().as_simple()));

        info!("Downloading repository {} to {:?}", repo, dir);
        self.download(url, &dir).await?;

        Ok(dir)
    }

    async fn download(&self, url: &str, dir: &Path) -> Result<()> {
        let dir = self.cache_dir.join(dir);

        if dir.exists() {
            warn!("Repository {} already exists in cache, skipping download", url);
            return Ok(());
        }

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&dir)?;

        // Use git clone to download the repository
        let output = Command::new("git")
            .args(["clone", url, "--depth", "1", "."])
            .current_dir(&dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::FailedToDownloadRepo(error.to_string()).into());
        }

        info!("Repository {} downloaded and stored at {}", url, dir.display());
        Ok(())
    }
}
