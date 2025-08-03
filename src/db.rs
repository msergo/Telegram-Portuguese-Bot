use sqlx::{Result, SqlitePool};
use std::fs;
use std::path::Path;

pub fn ensure_sqlite_file(path: &str) -> std::io::Result<()> {
    let db_path = Path::new(path);

    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?; // Create parent dir if needed
    }

    if !db_path.exists() {
        fs::File::create(db_path)?; // Create empty file
    }

    Ok(())
}
pub async fn init_db(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS cached_articles (
            id INTEGER PRIMARY KEY,
            word TEXT NOT NULL,
            lang_direction TEXT NOT NULL,
            html TEXT NOT NULL,
            formatted TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(word, lang_direction)
        );
        ",
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_cached_formatted(
    pool: &SqlitePool,
    word: &str,
    dir: &str,
) -> Result<Option<String>> {
    let row = sqlx::query_scalar::<_, String>(
        "SELECT formatted FROM cached_articles WHERE word = ? AND lang_direction = ? AND formatted IS NOT NULL"
    )
    .bind(word)
    .bind(dir)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn get_cached_html(pool: &SqlitePool, word: &str, dir: &str) -> Result<Option<String>> {
    let row = sqlx::query_scalar::<_, String>(
        "SELECT html FROM cached_articles WHERE word = ? AND lang_direction = ?",
    )
    .bind(word)
    .bind(dir)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn insert_html(pool: &SqlitePool, word: &str, dir: &str, html: &str) -> Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO cached_articles (word, lang_direction, html) VALUES (?, ?, ?)",
    )
    .bind(word)
    .bind(dir)
    .bind(html)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_formatted(
    pool: &SqlitePool,
    word: &str,
    dir: &str,
    formatted: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE cached_articles SET formatted = ?, updated_at = CURRENT_TIMESTAMP WHERE word = ? AND lang_direction = ?"
    )
    .bind(formatted)
    .bind(word)
    .bind(dir)
    .execute(pool)
    .await?;

    Ok(())
}
