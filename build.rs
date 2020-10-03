#[path = "src/version_data.rs"]
mod version_data;
use version_data::VersionData;

use chrono::Utc;
use std::env::{
    self,
    consts::{ARCH, OS},
};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() -> std::io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("version.json");
    let mut file = File::create(dest_path)?;

    let is_git_info_available = git_cmd()
        .arg("status")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let data = VersionData {
        build: env::var("PROFILE").unwrap().to_string(),
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        branch: if is_git_info_available {
            get_branch_name()
        } else {
            None
        }
        .unwrap_or("NONE".to_string()),
        commit: if is_git_info_available {
            get_commit_hash()
        } else {
            None
        }
        .unwrap_or("NONE".to_string()),
        clean_worktree: is_git_info_available && is_working_tree_clean(),
        os: OS.to_string(),
        arch: ARCH.to_string(),
        timestamp: Utc::now().format("%Y-%m-%d %H:%M").to_string(),
    };

    file.write(serde_json::to_string::<VersionData>(&data).unwrap().as_bytes())
        .unwrap();

    Ok(())
}

fn git_cmd<'a>() -> Command {
    let mut cmd = Command::new("git".to_string());
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    return cmd;
}

fn get_commit_hash() -> Option<String> {
    let output = git_cmd()
        .arg("log")
        .arg("-1")
        .arg("--pretty=format:%h")
        .output()
        .unwrap();

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

fn get_branch_name() -> Option<String> {
    let output = git_cmd()
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .unwrap();

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim_end().to_string())
    } else {
        None
    }
}

fn is_working_tree_clean() -> bool {
    let status = git_cmd()
        .arg("diff")
        .arg("--quiet")
        .arg("--exit-code")
        .status()
        .unwrap();

    status.success()
}
