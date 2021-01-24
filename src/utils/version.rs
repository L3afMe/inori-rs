use std::cmp::max;

use colored::Colorize;

use super::consts;
use crate::{inori_debug, inori_info};

pub async fn check_is_latest() {
    inori_info!("Updater", "Checking for update");
    let latest = get_version().await;

    inori_debug!(
        "Updater",
        "Comparing current version ({}) to latest tag ({})",
        consts::PROG_VERSION,
        latest
    );

    if compare_versions(&consts::PROG_VERSION, &latest) {
        inori_info!("Updater", "New update available at {}/releases", consts::GITHUB_LINK);
        inori_info!("Updater", "Current version: {}", consts::PROG_VERSION);
        inori_info!("Updater", "Available version: {}", latest);
    } else {
        inori_info!("Updater", "No new update found");
    }
}

async fn get_version() -> String {
    inori_debug!(
        "Updater",
        "Checking latest tag at https://api.github.com/repos/{}/{}/tags",
        consts::GITHUB_USERNAME,
        consts::GITHUB_REPO
    );

    let res = reqwest::Client::new()
        .get(&format!(
            "https://api.github.com/repos/{}/{}/tags",
            consts::GITHUB_USERNAME,
            consts::GITHUB_REPO
        ))
        .header("User-Agent", "Inori-rs")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    inori_debug!("Updater", "Got response from GitHub: {}", &res);

    let obj = serde_json::from_str::<serde_json::Value>(&res).unwrap();

    obj[0]["name"].as_str().unwrap().to_string()
}

fn split_version(version: &str) -> Vec<&str> {
    let mut version = version.strip_prefix('v').unwrap_or(version);

    if version.contains('~') {
        version = version.split('~').collect::<Vec<&str>>().get(0).unwrap();
    }

    version.split('.').collect::<Vec<&str>>()
}

fn compare_versions(curr_ver: &str, git_ver: &str) -> bool {
    let curr_split = split_version(curr_ver);
    let git_split = split_version(git_ver);

    for idx in 0..max(curr_split.len(), git_split.len()) {
        let curr_part = match curr_split.get(idx) {
            Some(curr_part) => curr_part,
            None => return true,
        };
        let git_part = match git_split.get(idx) {
            Some(git_part) => git_part,
            None => return false,
        };

        if curr_part != git_part {
            return git_part > curr_part;
        }
    }

    false
}
