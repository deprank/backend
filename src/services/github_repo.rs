use anyhow::{anyhow, Result};
use regex::Regex;
use std::{fs, path::PathBuf, process::Command};
use uuid::Uuid;

/// Download and store GitHub repository
///
/// # Arguments
/// * `github_url` - GitHub repository URL
///
/// # Returns
/// Returns the local path of the downloaded repository
pub async fn download_and_store_repo(github_url: &str) -> Result<PathBuf> {
    // Extract repository name from URL
    let repo_name = extract_repo_name(github_url)?;

    // Create project directory to store the repository
    let project_dir = PathBuf::from("project");
    fs::create_dir_all(&project_dir)?;

    // To avoid conflicts, create a subfolder with repository name + unique ID
    let repo_dir = project_dir.join(format!(
        "{}_{}",
        repo_name,
        Uuid::new_v4().to_string().split('-').next().unwrap_or("temp")
    ));
    fs::create_dir_all(&repo_dir)?;

    println!("Downloading repository {} to {:?}", github_url, repo_dir);

    // Use git clone to download the repository
    let output = Command::new("git")
        .args(["clone", github_url, "--depth", "1", "."])
        .current_dir(&repo_dir)
        .output()?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to download repository: {}", error_msg));
    }

    println!("Repository downloaded and stored at {:?}", repo_dir);

    // Return the path of the downloaded repository
    Ok(repo_dir)
}

/// Extract repository name from GitHub URL
fn extract_repo_name(github_url: &str) -> Result<String> {
    let re = Regex::new(r"github\.com/[^/]+/([^/\.]+)")?;

    if let Some(captures) = re.captures(github_url) {
        if let Some(name) = captures.get(1) {
            return Ok(name.as_str().to_string());
        }
    }

    // If unable to extract name, use a generic name
    Ok("github_repo".to_string())
}
