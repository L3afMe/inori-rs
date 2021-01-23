use once_cell::sync::Lazy;
use regex::Regex;

use crate::models::discord::Emote;

static MENTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"<@!?\d{18}>").unwrap());
static USER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(<@)?\d{18}>?").unwrap());
static CHANNEL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(<#)?\d{18}>?").unwrap());
static EMOTE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"<a?:[a-zA-Z0-9_]*?:\d{18}>").unwrap());
static EMOTE_NAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^:]{0,}[a-zA-Z0-9][^:]").unwrap());
static EMOTE_ID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d{18}").unwrap());

pub fn is_mention(arg: &str) -> bool {
    MENTION_REGEX.is_match(&arg)
}

pub fn is_user(arg: &str) -> bool {
    USER_REGEX.is_match(arg)
}

pub fn get_user(arg: &str) -> String {
    let mut arg = arg.strip_prefix('<').unwrap_or(arg);
    arg = arg.strip_prefix('@').unwrap_or(arg);
    arg = arg.strip_prefix('!').unwrap_or(arg);
    arg = arg.strip_suffix('>').unwrap_or(arg);

    arg.to_string()
}

pub fn is_channel(arg: &str) -> bool {
    CHANNEL_REGEX.is_match(arg)
}

pub fn get_channel(arg: &str) -> String {
    let mut arg = arg.strip_prefix('<').unwrap_or(arg);
    arg = arg.strip_prefix('#').unwrap_or(arg);
    arg = arg.strip_suffix('>').unwrap_or(arg);

    arg.to_string()
}

pub fn has_emotes(message: &str) -> bool {
    EMOTE_REGEX.is_match(&message)
}

pub fn get_emotes(arg: &str) -> Vec<Emote> {
    let mut matches = Vec::new();
    if !has_emotes(&arg) {
        return matches;
    }

    for mat in EMOTE_REGEX.captures_iter(&arg) {
        let mat = if let Some(mat) = mat.get(0) {
            mat.as_str()
        } else {
            continue;
        };

        if !EMOTE_ID_REGEX.is_match(&mat) || !EMOTE_NAME_REGEX.is_match(&mat) {
            continue;
        }

        let animated = mat.starts_with("<a:");

        let idm = EMOTE_ID_REGEX.find(&mat).unwrap();
        let id = mat[idm.start()..idm.end()].parse::<u64>().unwrap();

        let namem = EMOTE_NAME_REGEX.find(&mat).unwrap();
        let name = mat[namem.start()..namem.end()].to_string();

        let url = format!(
            "https://cdn.discordapp.com/emojis/{}.{}",
            id,
            if animated { "gif" } else { "png" }
        );

        let emote = Emote {
            name: name.to_string(),
            id,
            url,
            animated,
        };

        if !matches.contains(&emote) {
            matches.push(emote);
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_mention() {
        // Valid ID
        assert_eq!(is_mention("<@779273941402255360>"), true);

        // Too short
        assert_eq!(is_mention("<@779241402255360>"), false);

        // Too long
        assert_eq!(is_mention("<@77924140225536012423234>"), false);

        // Not an ID at all
        assert_eq!(is_mention("This isn't an ID"), false);
    }

    #[test]
    fn test_has_emotes() {
        // Valid emote
        assert_eq!(has_emotes("<:cccatgirl:800141155424927785>"), true);

        // User ID
        assert_eq!(has_emotes("<@779273941402255360>"), false);

        // Random text
        assert_eq!(has_emotes("I'm not an emote >:("), false);
    }

    #[test]
    fn test_get_emotes() {
        let emotes = get_emotes(
            "<:cccatgirl:800141155424927785><:cccatboy:800141190338707466><:HUGERS:785150570591551491>
            <a:z_:800797540739579924>",
        );

        // Valid length
        assert_eq!(emotes.len(), 4);

        // Not animated
        assert_eq!(emotes.get(0).unwrap().animated, false);
        assert_eq!(emotes.get(1).unwrap().animated, false);
        assert_eq!(emotes.get(2).unwrap().animated, false);

        // Animated
        assert_eq!(emotes.get(3).unwrap().animated, true);

        // Getting name
        assert_eq!(emotes.get(0).unwrap().name, "cccatgirl");
        assert_eq!(emotes.get(1).unwrap().name, "cccatboy");
        assert_eq!(emotes.get(2).unwrap().name, "HUGERS");
        assert_eq!(emotes.get(3).unwrap().name, "z_");

        // Getting ID
        assert_eq!(emotes.get(0).unwrap().id, 800141155424927785);
        assert_eq!(emotes.get(1).unwrap().id, 800141190338707466);
        assert_eq!(emotes.get(2).unwrap().id, 785150570591551491);
        assert_eq!(emotes.get(3).unwrap().id, 800797540739579924);
    }
}
