use std::collections::HashMap;

use async_trait::async_trait;
use serde_derive::Deserialize;
use serenity::{
    builder::{CreateEmbed, CreateMessage},
    framework::standard::CommandError,
    http::AttachmentType,
    model::{channel::Message, id::ChannelId, Permissions},
    prelude::Context,
    utils::Colour,
};
use serenity_utils::menu::{Menu, MenuOptions};
use tokio::time::{delay_for, Duration};

use crate::{utils::general::get_perms, Settings};

#[derive(Clone, Debug, Deserialize)]
pub struct BasicUser {
    pub username:      String,
    pub discriminator: String,
}

pub struct Emote {
    pub name:     String,
    pub id:       u64,
    pub url:      String,
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
            name:     (&self.name).to_string(),
            id:       self.id,
            url:      (&self.url).to_string(),
            animated: self.animated,
        }
    }
}

impl Emote {
    pub fn new(id: u64, name: String, animated: bool) -> Emote {
        let ext = if animated { "gif" } else { "png" };

        Emote {
            id,
            name,
            url: format!("https://cdn.discordapp.com/emojis/{}.{}", id, ext),
            animated,
        }
    }

    fn to_message(&self) -> String {
        let a = if self.animated { "a" } else { "" };

        format!("<{}:{}:{}>", a, self.name, self.id)
    }
}

#[derive(Debug, Clone)]
pub struct MessageField {
    title:   String,
    content: String,
    inline:  bool,
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

#[derive(Debug, Clone)]
pub struct MessageCreator<'a> {
    title:        Option<String>,
    mode:         u64,
    content:      Option<String>,
    image:        Option<String>,
    attachment:   Option<AttachmentType<'a>>,
    thumbnail:    Option<String>,
    fields:       Vec<MessageField>,
    footer_text:  Option<String>,
    footer_image: Option<String>,
    colour:       Option<Colour>,
}

impl<'a> Default for MessageCreator<'a> {
    fn default() -> MessageCreator<'a> {
        MessageCreator {
            title:        None,
            mode:         0,
            content:      None,
            image:        None,
            attachment:   None,
            thumbnail:    None,
            fields:       Vec::new(),
            footer_text:  None,
            footer_image: None,
            colour:       None,
        }
    }
}

impl<'a> MessageCreator<'a> {
    pub fn to_message(&self, emotes: HashMap<String, u64>) -> CreateMessage {
        let mut message = CreateMessage::default();
        let mut ctnt = String::new();

        if let Some(title) = &self.title {
            ctnt = format!("[{}]", title);
        }

        if let Some(content) = &self.content {
            let content = if self.mode == 0 {
                content.to_string()
            } else {
                let emote_name = match self.mode {
                    1 => "loading",
                    2 => "response_success",
                    3 => "response_info",
                    4 => "response_warning",
                    _ => "response_error",
                };

                let emote = if let Some(emote) = emotes.get(emote_name) {
                    if self.mode == 1 {
                        format!("<a:{}:{}> ", emote_name, emote)
                    } else {
                        format!("<:{}:{}> ", emote_name, emote)
                    }
                } else {
                    match self.mode {
                        1 => "Loading: ",
                        2 => "Success: ",
                        3 => "Info: ",
                        4 => "Warning: ",
                        _ => "Error: ",
                    }
                    .to_string()
                };

                format!("{} {}", emote, content)
            };

            ctnt = format!("{}\n{}", ctnt, content);
        }

        for field in &self.fields {
            ctnt = format!("{}\n_[_**{}**_]_\n{}", ctnt, field.title, field.content);
        }

        if let Some(footer_text) = &self.footer_text {
            ctnt = format!("{}\n_{}_", ctnt, footer_text);
        }

        // Append image URL to the end of the message
        if let Some(image) = &self.image {
            ctnt = format!("{}\n{}", ctnt, image);
        }

        message.content(ctnt).clone()
    }

    pub fn to_embed(&self, emotes: HashMap<String, u64>) -> CreateMessage {
        let mut message = CreateMessage::default();

        message.embed(|e: &mut CreateEmbed| {
            if let Some(colour) = self.colour {
                e.colour(colour);
            } else {
                e.colour(match self.mode {
                    0 => Colour::FABLED_PINK,
                    1 => Colour::BLURPLE,
                    2 => Colour::FOOYOO,
                    3 => Colour::KERBAL,
                    4 => Colour::ORANGE,
                    _ => Colour::MEIBE_PINK,
                });
            }

            if let Some(title) = &self.title {
                e.title(format!("[{}]", title));
            }

            if let Some(content) = &self.content {
                let emote = if self.mode == 0 {
                    String::default()
                } else {
                    let emote_name = match self.mode {
                        1 => "loading",
                        2 => "response_success",
                        3 => "response_info",
                        4 => "response_warning",
                        _ => "response_error",
                    };

                    if let Some(emote) = emotes.get(emote_name) {
                        if self.mode == 1 {
                            format!("<a:{}:{}> ", emote_name, emote)
                        } else {
                            format!("<:{}:{}> ", emote_name, emote)
                        }
                    } else {
                        match self.mode {
                            1 => "Loading: ",
                            2 => "Success: ",
                            3 => "Info: ",
                            4 => "Warning: ",
                            _ => "Error: ",
                        }
                        .to_string()
                    }
                };

                e.description(format!("{} {}", emote, content));
            }

            if let Some(image) = &self.image {
                e.image(image);
            }

            if let Some(thumbname) = &self.thumbnail {
                e.thumbnail(thumbname);
            }

            for field in &self.fields {
                e.field(field.title.clone(), field.content.clone(), field.inline);
            }

            e.footer(|f| {
                if let Some(image) = &self.footer_image {
                    f.icon_url(image);
                }

                if let Some(text) = &self.footer_text {
                    f.text(text);
                }

                f
            });

            e
        });

        message
    }

    pub fn to_auto(&self, perms: Permissions, emotes: HashMap<String, u64>) -> CreateMessage {
        if perms.embed_links() {
            self.to_embed(emotes)
        } else {
            self.to_message(emotes)
        }
    }

    pub fn title<D: ToString>(&mut self, title: D) -> &mut Self {
        self.title = Some(title.to_string());

        self
    }

    pub fn content<D: ToString>(&mut self, content: D) -> &mut Self {
        self.content = Some(content.to_string());

        self
    }

    pub fn loading(&mut self) -> &mut Self {
        self.mode = 1;

        self
    }

    pub fn success(&mut self) -> &mut Self {
        self.mode = 2;

        self
    }

    pub fn info(&mut self) -> &mut Self {
        self.mode = 3;

        self
    }

    pub fn warning(&mut self) -> &mut Self {
        self.mode = 4;

        self
    }

    pub fn error(&mut self) -> &mut Self {
        self.mode = 5;

        self
    }

    pub fn image<D: ToString>(&mut self, url: D) -> &mut Self {
        self.image = Some(url.to_string());

        self
    }

    pub fn attachment<T: Into<AttachmentType<'a>>>(&mut self, attachment: T) -> &mut Self {
        self.attachment = Some(attachment.into());

        self
    }

    pub fn thumbnail<D: ToString>(&mut self, url: D) -> &mut Self {
        self.thumbnail = Some(url.to_string());

        self
    }

    pub fn field<D: ToString, T: ToString>(&mut self, title: D, content: T, inline: bool) -> &mut Self {
        let field = MessageField::new(&title.to_string(), &content.to_string(), inline);

        self.fields.push(field);

        self
    }

    pub fn footer_image<D: ToString>(&mut self, url: D) -> &mut Self {
        self.footer_image = Some(url.to_string());

        self
    }

    pub fn footer_text<D: ToString>(&mut self, text: D) -> &mut Self {
        self.footer_text = Some(text.to_string());

        self
    }

    pub fn colour(&mut self, colour: Colour) -> &mut Self {
        self.colour = Some(colour);

        self
    }
}

#[async_trait]
pub trait InoriChannelUtils {
    async fn send_tmp<'a, F: std::marker::Send>(&self, ctx: &Context, f: F) -> Result<(), CommandError>
    where
        for<'b> F: FnOnce(&'b mut MessageCreator<'a>) -> &'b mut MessageCreator<'a>;

    async fn send_noret<'a, F: std::marker::Send>(&self, ctx: &Context, f: F) -> Result<(), CommandError>
    where
        for<'b> F: FnOnce(&'b mut MessageCreator<'a>) -> &'b mut MessageCreator<'a>;

    async fn send<'a, F: std::marker::Send>(&self, ctx: &Context, f: F) -> Result<Message, CommandError>
    where
        for<'b> F: FnOnce(&'b mut MessageCreator<'a>) -> &'b mut MessageCreator<'a>;

    async fn send_loading<D: ToString + std::marker::Send>(
        &self,
        ctx: &Context,
        title: D,
        loading_msg: &str,
    ) -> Result<Message, CommandError>;

    async fn send_paginator<'a>(
        &self,
        ctx: &Context,
        msg: &Message,
        embeds: Vec<MessageCreator<'a>>,
    ) -> Result<Option<Message>, CommandError>;

    async fn send_paginator_noret<'a>(
        &self,
        ctx: &Context,
        msg: &Message,
        embeds: Vec<MessageCreator<'a>>,
    ) -> Result<(), CommandError>;

    async fn send_paginatorwo<'a>(
        &self,
        ctx: &Context,
        msg: &Message,
        embeds: Vec<MessageCreator<'a>>,
        options: MenuOptions,
    ) -> Result<Option<Message>, CommandError>;

    async fn send_paginatorwo_noret<'a>(
        &self,
        ctx: &Context,
        msg: &Message,
        embeds: Vec<MessageCreator<'a>>,
        options: MenuOptions,
    ) -> Result<(), CommandError>;
}

#[async_trait]
impl InoriChannelUtils for ChannelId {
    async fn send_tmp<'a, F: std::marker::Send>(&self, ctx: &Context, f: F) -> Result<(), CommandError>
    where
        for<'b> F: FnOnce(&'b mut MessageCreator<'a>) -> &'b mut MessageCreator<'a>, {
        match self.send(ctx, f).await {
            Ok(msg) => msg.autodelete(ctx).await,
            Err(err) => return Err(err),
        }
    }

    async fn send_noret<'a, F: std::marker::Send>(&self, ctx: &Context, f: F) -> Result<(), CommandError>
    where
        for<'b> F: FnOnce(&'b mut MessageCreator<'a>) -> &'b mut MessageCreator<'a>, {
        match self.send(ctx, f).await {
            Ok(_) => Ok(()),
            Err(err) => return Err(err),
        }
    }

    async fn send<'a, F: std::marker::Send>(&self, ctx: &Context, f: F) -> Result<Message, CommandError>
    where
        for<'b> F: FnOnce(&'b mut MessageCreator<'a>) -> &'b mut MessageCreator<'a>, {
        let mut msg_creator = MessageCreator::default();
        let msg = f(&mut msg_creator);
        let perms = get_perms(ctx, self).await;
        let emotes = {
            // TODO: Check if has nitro
            if let Ok(_user) = ctx.http.get_current_user().await {
                let data = ctx.data.read().await;
                let settings = data.get::<Settings>().expect("Expected Settings in TypeMap.").lock().await;

                settings.sb_emotes.clone()
            } else {
                HashMap::new()
            }
        };

        let res = self
            .send_message(&ctx, |m| {
                m.0 = msg.to_auto(perms, emotes).0;

                m
            })
            .await;

        match res {
            Ok(msg) => Ok(msg),
            Err(why) => Err(CommandError::from(why)),
        }
    }

    async fn send_loading<D: ToString + std::marker::Send>(
        &self,
        ctx: &Context,
        title: D,
        loading_msg: &str,
    ) -> Result<Message, CommandError> {
        self.send(ctx, |f: &mut MessageCreator| {
            f.loading().title(title).content(&format!("{}...", loading_msg))
        })
        .await
    }

    async fn send_paginator<'a>(
        &self,
        ctx: &Context,
        msg: &Message,
        embeds: Vec<MessageCreator<'a>>,
    ) -> Result<Option<Message>, CommandError> {
        self.send_paginatorwo(ctx, msg, embeds, MenuOptions::default()).await
    }

    async fn send_paginator_noret<'a>(
        &self,
        ctx: &Context,
        msg: &Message,
        embeds: Vec<MessageCreator<'a>>,
    ) -> Result<(), CommandError> {
        match self.send_paginator(ctx, msg, embeds).await {
            Ok(_) => Ok(()),
            Err(why) => Err(why),
        }
    }

    async fn send_paginatorwo<'a>(
        &self,
        ctx: &Context,
        msg: &Message,
        embeds: Vec<MessageCreator<'a>>,
        options: MenuOptions,
    ) -> Result<Option<Message>, CommandError> {
        let perms = get_perms(ctx, &msg.channel_id).await;
        let mut formatted_embeds = Vec::new();
        let emotes = {
            let data = ctx.data.read().await;
            let settings = data.get::<Settings>().expect("Expected Settings in TypeMap").lock().await;

            settings.sb_emotes.clone()
        };

        for (idx, embed) in embeds.iter().enumerate() {
            let mut msg = CreateMessage::default();
            let mut embed = embed.clone();
            embed.footer_text(format!("Page {} of {}", idx + 1, embeds.len()));

            msg.0 = embed.to_auto(perms, emotes.clone()).0;
            formatted_embeds.push(msg);
        }

        let res = Menu::new(ctx, msg, &formatted_embeds[..], options).run().await;

        match res {
            Ok(msg) => Ok(msg),
            Err(why) => Err(CommandError::from(why)),
        }
    }

    async fn send_paginatorwo_noret<'a>(
        &self,
        ctx: &Context,
        msg: &Message,
        embeds: Vec<MessageCreator<'a>>,
        options: MenuOptions,
    ) -> Result<(), CommandError> {
        match self.send_paginatorwo(ctx, msg, embeds, options).await {
            Ok(_) => Ok(()),
            Err(why) => Err(why),
        }
    }
}

#[async_trait]
pub trait InoriMessageUtils {
    async fn autodelete(&self, ctx: &Context) -> Result<(), CommandError>;

    async fn update_tmp<'a, F: std::marker::Send>(&'a mut self, ctx: &Context, f: F) -> Result<(), CommandError>
    where
        for<'b> F: FnOnce(&'b mut MessageCreator<'a>) -> &'b mut MessageCreator<'a>;

    async fn update_noret<'a, F: std::marker::Send>(&'a mut self, ctx: &Context, f: F) -> Result<(), CommandError>
    where
        for<'b> F: FnOnce(&'b mut MessageCreator<'a>) -> &'b mut MessageCreator<'a>;

    async fn update<'a, F: std::marker::Send>(&'a mut self, ctx: &Context, f: F) -> Result<&'a Message, CommandError>
    where
        for<'b> F: FnOnce(&'b mut MessageCreator<'a>) -> &'b mut MessageCreator<'a>;
}

#[async_trait]
impl InoriMessageUtils for Message {
    async fn autodelete(&self, ctx: &Context) -> Result<(), CommandError> {
        let ad_delay = {
            let data = ctx.data.read().await;
            let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

            if settings.autodelete.enabled {
                Some(settings.autodelete.delay)
            } else {
                None
            }
        };

        if let Some(delay) = ad_delay {
            let ctx = ctx.clone();
            let msg = self.clone();

            tokio::task::spawn(async move {
                delay_for(Duration::from_secs(delay)).await;

                let _ = ctx.http.delete_message(msg.channel_id.0, msg.id.0).await;
            });
        }

        Ok(())
    }

    async fn update_tmp<'a, F: std::marker::Send>(&'a mut self, ctx: &Context, f: F) -> Result<(), CommandError>
    where
        for<'b> F: FnOnce(&'b mut MessageCreator<'a>) -> &'b mut MessageCreator<'a>, {
        match self.update(ctx, f).await {
            Ok(msg) => msg.autodelete(ctx).await,
            Err(err) => return Err(err),
        }
    }

    async fn update_noret<'a, F: std::marker::Send>(&'a mut self, ctx: &Context, f: F) -> Result<(), CommandError>
    where
        for<'b> F: FnOnce(&'b mut MessageCreator<'a>) -> &'b mut MessageCreator<'a>, {
        match self.update(ctx, f).await {
            Ok(_) => Ok(()),
            Err(err) => return Err(err),
        }
    }

    async fn update<'a, F: std::marker::Send>(&'a mut self, ctx: &Context, f: F) -> Result<&'a Message, CommandError>
    where
        for<'b> F: FnOnce(&'b mut MessageCreator<'a>) -> &'b mut MessageCreator<'a>, {
        let mut msg_creator = MessageCreator::default();
        let msg = f(&mut msg_creator);
        let perms = get_perms(ctx, &self.channel_id).await;
        let emotes = {
            let data = ctx.data.read().await;
            let settings = data.get::<Settings>().expect("Expected Settings in TypeMap.").lock().await;
            settings.sb_emotes.clone()
        };

        let res = self
            .edit(&ctx.http, |m| {
                m.0 = msg.to_auto(perms, emotes).0;

                m
            })
            .await;

        match res {
            Ok(_) => Ok(self),
            Err(why) => Err(CommandError::from(why)),
        }
    }
}
