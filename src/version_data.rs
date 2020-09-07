use serde::{
    Serialize,
    Deserialize,
};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct VersionData {
    pub build: String,
    pub name: String,
    pub version: String,
    pub branch: String,
    pub commit: String,
    pub clean_worktree: bool,
    pub os: String,
    pub arch: String,
    pub timestamp: String,
}

impl fmt::Display for VersionData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} ({}:{}{}, {} build, {} [{}], {})", self.name, self.version, self.branch, self.commit, if self.clean_worktree { "" } else { "*" }, self.build, self.os, self.arch, self.timestamp)
    }
}

