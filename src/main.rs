use dotenv::dotenv;
use teloxide::{prelude::*, types::ParseMode, update_listeners::webhooks};

mod fetch_translations;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting Portuguese dict bot...");

    let bot = Bot::from_env();

    let addr = ([127, 0, 0, 1], 3030).into();
    let webhook_url = std::env::var("WEBHOOK_URL").expect("WEBHOOK_URL must be set");
    let url = reqwest::Url::parse(&webhook_url).expect("Invalid WEBHOOK_URL");

    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await
        .expect("Couldn't setup webhook");

    teloxide::repl_with_listener(
        bot,
        |bot: Bot, msg: Message| async move {
            // bot.send_message(msg.chat.id, "pong").await?;
            let message = fetch_translations::fetch(&msg.text().unwrap_or("")).await;
            bot.send_message(msg.chat.id, message)
                .parse_mode(ParseMode::Html)
                .await?;

            Ok(())
        },
        listener,
    )
    .await;
}
