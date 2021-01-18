use serde_derive::Deserialize;
use serde_json::Value;
use serenity::builder::CreateMessage;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct BasicUser {
    pub username: String,
    pub discriminator: String,
}

pub struct Emote {
    pub name: String,
    pub id: u64,
    pub url: String,
    pub animated: bool,
}

impl PartialEq for Emote {
    fn eq(&self, other: &Emote) -> bool {
        self.id == other.id
    }
}

impl Clone for Emote {
    fn clone(&self) -> Emote {
        Emote {
            name: (&self.name).to_string(),
            id: self.id.clone(),
            url: (&self.url).to_string(),
            animated: self.animated.clone(),
        }
    }
}

pub struct MessageField {
    title: String,
    content: String,
    inline: bool,
}

impl MessageField {
    fn new(title: &str, content: &str, inline: bool) -> Self {
        MessageField {
            title: title.to_string(),
            content: content.to_string(),
            inline,
        }
    }
}

pub struct MessageCreator<'a> {
    title: &'a str,
    content: Option<&'a str>,
    image: Option<&'a str>,
    fields: Vec<MessageField>,
}

impl<'a> MessageCreator<'a> {
    pub const fn new(title: &'a str) -> Self {
        MessageCreator {
            title,
            content: None,
            image: None,
            fields: Vec::new(),
        }
    }

    pub fn to_embed(&self) -> HashMap<&'static str, Value> {
        let mut message = CreateMessage::default();
        message.embed(|e| {
            e.title(format!("[{}]", &self.title));
            e
        });

        message.0
    }
}
