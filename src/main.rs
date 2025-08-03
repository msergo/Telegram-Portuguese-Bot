use dotenv::dotenv;
use teloxide::{prelude::*, types::ParseMode};

mod fetch_translations;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting Portuguese dict bot...");

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        let message = fetch_translations::fetch(&msg.text().unwrap_or("")).await;
        bot.send_message(msg.chat.id, message)
            .parse_mode(ParseMode::Html)
            .await?;
        Ok(())
    })
    .await;
}
