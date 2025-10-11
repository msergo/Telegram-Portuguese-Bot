use std::sync::Arc;

use dotenv::dotenv;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use sqlx::SqlitePool;
use teloxide::{
    prelude::*,
    types::{ChatKind, ParseMode},
    update_listeners::webhooks,
};
mod db;
mod fetch_translations;
use pt_dict_bot::migration::Migrator;
use pt_dict_bot::user_repository::UserRepository;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting Portuguese dict bot...");

    // Initialize the database (sqlx pool)
    db::ensure_sqlite_file("./cache/translations.db").expect("Failed to ensure SQLite file exists");
    let pool = SqlitePool::connect("sqlite://cache/translations.db")
        .await
        .expect("Failed to open database");
    db::init_db(&pool)
        .await
        .expect("Failed to initialize database");

    // Use a separate SeaORM connection for user configs (we keep two connectors for now)
    let sea_orm_db = sea_orm::Database::connect("sqlite://cache/translations.db")
        .await
        .expect("Failed to connect to database with SeaORM");

    // Run migrations
    Migrator::up(&sea_orm_db, None)
        .await
        .expect("Failed to run migrations");

    let user_repo = UserRepository::new(sea_orm_db);

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
            let user_repo = user_repo.clone(); // clone the user repository

            async move {
                // Log chat ID and message ID for debugging
                log::info!("Received message in chat {}", msg.chat.id);
                let word = msg.text().unwrap_or("").trim().to_string().to_lowercase();
                // If word is empty, do nothing
                if word.is_empty() {
                    return Ok(());
                }

                let bot_name = bot.get_me().await.unwrap().username.clone();
                let bot_name_str = format!("@{}", bot_name.as_deref().unwrap_or(""));
                // If it is a message in the group and the message is NOT addressed to the bot, do nothing
                if matches!(msg.chat.kind, ChatKind::Public(_)) && !word.starts_with(&bot_name_str)
                {
                    return Ok(());
                }

                // If it is a message in the group and the message is addressed to the bot, remove the bot's name from the word
                let word = if matches!(msg.chat.kind, ChatKind::Public(_))
                    && word.starts_with(&bot_name_str)
                {
                    word.replacen(&bot_name_str, "", 1).trim().to_string()
                } else {
                    word
                };

                // Get translation direction from database with "pten" default fallback
                // Note: chat_id represents chat context (group ID for groups, user ID for private chats)
                let chat_id = Arc::new(msg.chat.id.to_string().replace("-", ""));
                let chat_translation_direction = match user_repo.get_user(&chat_id).await {
                    Ok(Some(user)) => user.translation_direction,
                    _ => "pten".to_string(), // Default fallback
                };

                // Check if cached in DB
                if let Some(cached) =
                    db::get_cached_formatted(&pool, &word, &chat_translation_direction)
                        .await
                        .unwrap()
                {
                    bot.send_message(msg.chat.id, cached)
                        .parse_mode(ParseMode::Html)
                        .await?;
                    return Ok(());
                }

                // TODO: Refactor this huge if statement
                // check if cached raw HTML exists without formatted translation (for cases when it was removed when formatting has changed)
                if let Some(cached_html) =
                    db::get_cached_html(&pool, &word, &chat_translation_direction)
                        .await
                        .unwrap()
                {
                    let translations = fetch_translations::get_translations(&cached_html);
                    // Store the formatted translation in the database
                    let _ = db::update_formatted(
                        &pool,
                        &word,
                        &chat_translation_direction,
                        &translations,
                    )
                    .await;
                    bot.send_message(msg.chat.id, translations)
                        .parse_mode(ParseMode::Html)
                        .await?;
                    return Ok(());
                }

                // Not cached, fetch
                let body = fetch_translations::fetch(&word, &chat_translation_direction).await;

                let raw_translations =
                    fetch_translations::get_raw_translations(&body, &chat_translation_direction);

                if raw_translations.is_empty() {
                    bot.send_message(msg.chat.id, "No translations found.")
                        .parse_mode(ParseMode::Html)
                        .await?;
                    return Ok(());
                }

                /*
                Store the fetched HTML in the database
                */

                let translations = fetch_translations::get_translations(&raw_translations);

                if translations.is_empty() {
                    bot.send_message(msg.chat.id, "No translations found.")
                        .parse_mode(ParseMode::Html)
                        .await?;
                    return Ok(());
                }

                // TODO: store in one go
                let _ =
                    db::insert_html(&pool, &word, &chat_translation_direction, &raw_translations)
                        .await;
                let _ =
                    db::update_formatted(&pool, &word, &chat_translation_direction, &translations)
                        .await;

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
