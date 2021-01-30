use colored::Colorize;
use semver::Version;

use super::consts;
use crate::{inori_debug, inori_error, inori_info};

pub async fn check_is_latest() {
    inori_info!("Updater", "Checking for update");
    let latest = get_version().await;

    let latest = if let Ok(ver) = Version::parse(&latest) {
        ver
    } else {
        inori_error!("Updater", "Unable to parse latest semver: '{}'", latest);
        return;
    };

    let current = if let Ok(ver) = Version::parse(&consts::PROG_VERSION) {
        ver
    } else {
        inori_error!("Updater", "Unable to parse current semver: '{}'", latest);
        return;
    };

    inori_debug!(
        "Updater",
        "Comparing current version ({}) to latest tag ({})",
        consts::PROG_VERSION,
        latest
    );


    if current < latest {
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
