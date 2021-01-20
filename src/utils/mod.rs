pub mod chat;
pub mod checks;
pub mod consts;
pub mod emotes;
pub mod general;
pub mod macros;
pub mod user;

use std::cmp::max;

fn split_version(version: &str) -> Vec<&str> {
    let mut version = if version.starts_with("v") {
        &version[1..]
    } else {
        version
    };

    if version.contains("~") {
        version = version.split("~").collect::<Vec<&str>>().get(0).unwrap();
    }

    version.split(".").collect::<Vec<&str>>()
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
