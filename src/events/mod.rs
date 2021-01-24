pub mod chat;
pub mod error;
pub mod help;

use std::{fs::DirEntry, io::Error, path::Path, sync::Arc};

use colored::Colorize;
use rand::Rng;
use serenity::{async_trait, model::gateway::Ready, prelude::*, utils::read_image};
use tokio::{
    task,
    time::{delay_for, Duration},
};

use crate::{inori_info, inori_success, models::settings::Settings, utils::consts};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        inori_success!(
            "Bot",
            "{} started, connected as {}#{:0>4}",
            consts::PROG_NAME,
            ready.user.name,
            ready.user.discriminator,
        );

        spawn_pfp_change_thread(Arc::new(Mutex::new(ctx))).await;
    }
}

async fn spawn_pfp_change_thread(ctx: Arc<Mutex<Context>>) {
    task::spawn(async move {
        loop {
            let start_time = std::time::SystemTime::now();
            loop {
                {
                    let ctx = ctx.lock().await;
                    let data = ctx.data.read().await;
                    let settings = data.get::<Settings>().expect("Expected Setting in TypeMap.").lock().await;

                    if settings.pfp_switcher.enabled
                        && start_time.elapsed().unwrap().as_secs() >= (settings.pfp_switcher.delay * 60) as u64
                    {
                        let path = Path::new("./pfps/");
                        if path.exists() {
                            let ops = path.read_dir().unwrap().collect::<Vec<Result<DirEntry, Error>>>();
                            let new_pfp = match settings.pfp_switcher.mode {
                                0 => ops[rand::thread_rng().gen_range(0..ops.len())].as_ref(),
                                1 => {
                                    // TODO: This shit
                                    ops[rand::thread_rng().gen_range(0..ops.len())].as_ref()
                                },
                                _ => ops[rand::thread_rng().gen_range(0..ops.len())].as_ref(),
                            }
                            .unwrap();

                            let mut user = ctx.cache.current_user().await;
                            let avatar =
                                read_image(format!("./pfps/{}", new_pfp.file_name().into_string().unwrap())).unwrap();
                            user.edit(&ctx.http, |p| p.avatar(Some(&avatar))).await.unwrap();

                            inori_info!("PfpSwitcher", "Changing pfps");
                            break;
                        }
                    }
                }

                delay_for(Duration::from_secs(60)).await;
            }
        }
    });
}
