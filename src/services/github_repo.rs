use std::path::PathBuf;
use std::process::Command;
use std::fs;
use anyhow::{Result, anyhow};
use uuid::Uuid;
use regex::Regex;

// 添加analyzer模块引用
use crate::services::analyzer;

/// 下载并存储 GitHub 仓库
/// 
/// # 参数
/// * `github_url` - GitHub 仓库的 URL
/// 
/// # 返回
/// 返回下载的仓库在本地的路径
pub async fn download_and_store_repo(github_url: &str) -> Result<PathBuf> {
    // 从URL中提取仓库名称
    let repo_name = extract_repo_name(github_url)?;
    
    // 创建项目的project文件夹用于存储仓库
    let project_dir = PathBuf::from("project");
    fs::create_dir_all(&project_dir)?;
    
    // 为了避免冲突，使用仓库名+唯一ID创建子文件夹
    let repo_dir = project_dir.join(format!("{}_{}", repo_name, Uuid::new_v4().to_string().split('-').next().unwrap_or("temp")));
    fs::create_dir_all(&repo_dir)?;
    
    println!("正在下载仓库 {} 到 {:?}", github_url, repo_dir);
    
    // 使用 git clone 下载仓库
    let output = Command::new("git")
        .args(&["clone", github_url, "--depth", "1", "."])
        .current_dir(&repo_dir)
        .output()?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("下载仓库失败: {}", error_msg));
    }
    
    println!("仓库下载完成，存储在 {:?}", repo_dir);
    
    // 返回下载的仓库路径
    Ok(repo_dir)
}

/// 从GitHub URL中提取仓库名称
fn extract_repo_name(github_url: &str) -> Result<String> {
    let re = Regex::new(r"github\.com/[^/]+/([^/\.]+)")?;
    
    if let Some(captures) = re.captures(github_url) {
        if let Some(name) = captures.get(1) {
            return Ok(name.as_str().to_string());
        }
    }
    
    // 如果无法提取名称，使用通用名称
    Ok("github_repo".to_string())
}



