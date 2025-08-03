use dotenv::dotenv;
use sqlx::SqlitePool;
use teloxide::{prelude::*, types::ParseMode, update_listeners::webhooks};
mod db;
mod fetch_translations;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting Portuguese dict bot...");

    // Initialize the database
    db::ensure_sqlite_file("./cache/translations.db").expect("Failed to ensure SQLite file exists");
    let pool = SqlitePool::connect("sqlite://cache/translations.db")
        .await
        .expect("Failed to open database");
    db::init_db(&pool)
        .await
        .expect("Failed to initialize database");

    let bot = Bot::from_env();

    let addr = ([127, 0, 0, 1], 3030).into();
    let webhook_address = std::env::var("WEBHOOK_ADDRESS").expect("WEBHOOK_ADDRESS must be set");
    let url = reqwest::Url::parse(&webhook_address).expect("Invalid WEBHOOK_ADDRESS");

    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await
        .expect("Couldn't setup webhook");

    teloxide::repl_with_listener(
        bot,
        move |bot: Bot, msg: Message| {
            let pool = pool.clone(); // clone the pool for this handler

            async move {
                let word = msg.text().unwrap_or("").trim().to_string();

                // Check if cached in DB
                if let Some(cached) = db::get_cached_formatted(&pool, &word, "pten")
                    .await
                    .unwrap()
                {
                    bot.send_message(msg.chat.id, cached)
                        .parse_mode(ParseMode::Html)
                        .await?;
                    return Ok(());
                }

                // TODO: declare enum for language direction
                // TODO: Refactor this huge if statement

                // check if cached raw HTML exists without formatted translation (for cases when it was removed when formatting has changed)
                if let Some(cached_html) = db::get_cached_html(&pool, &word, "pten").await.unwrap()
                {
                    let translations = fetch_translations::parse_body(&cached_html).await;
                    // Store the formatted translation in the database
                    let _ = db::update_formatted(&pool, &word, "pten", &translations).await;
                    bot.send_message(msg.chat.id, translations)
                        .parse_mode(ParseMode::Html)
                        .await?;
                    return Ok(());
                }

                // Not cached, fetch
                let body = fetch_translations::fetch(&word).await;

                /*
                Store the fetched HTML in the database
                */

                let translations = fetch_translations::parse_body(&body).await;
                if translations.is_empty() {
                    bot.send_message(msg.chat.id, "No translations found.")
                        .parse_mode(ParseMode::Html)
                        .await?;
                    return Ok(());
                }

                let _ = db::insert_html(&pool, &word, "pten", &body).await;
                let _ = db::update_formatted(&pool, &word, "pten", &translations).await;

                bot.send_message(msg.chat.id, translations)
                    .parse_mode(ParseMode::Html)
                    .await?;

                Ok(())
            }
        },
        listener,
    )
    .await;
}
