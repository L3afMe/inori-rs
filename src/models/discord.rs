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
