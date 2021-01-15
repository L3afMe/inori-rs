use serenity::model::user::User;

pub async fn get_av(user: &User) -> String {
    match user.avatar_url() {
        Some(av) => av,
        None => user.default_avatar_url(),
    }
}
