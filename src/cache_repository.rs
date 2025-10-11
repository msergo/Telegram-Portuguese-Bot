use crate::entities::cached_articles::{self, ActiveModel, Entity as CachedArticles};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

#[derive(Clone)]
pub struct CacheRepository {
    db: DatabaseConnection,
}

impl CacheRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_cached_formatted(
        &self,
        word: &str,
        dir: &str,
    ) -> Result<Option<String>, sea_orm::DbErr> {
        let res = CachedArticles::find()
            .filter(cached_articles::Column::Word.eq(word.to_string()))
            .filter(cached_articles::Column::LangDirection.eq(dir.to_string()))
            .one(&self.db)
            .await?;

        Ok(res.and_then(|m| m.formatted))
    }

    pub async fn get_cached_html(
        &self,
        word: &str,
        dir: &str,
    ) -> Result<Option<String>, sea_orm::DbErr> {
        let res = CachedArticles::find()
            .filter(cached_articles::Column::Word.eq(word.to_string()))
            .filter(cached_articles::Column::LangDirection.eq(dir.to_string()))
            .one(&self.db)
            .await?;

        Ok(res.map(|m| m.html))
    }

    pub async fn insert_html(
        &self,
        word: &str,
        dir: &str,
        html: &str,
    ) -> Result<(), sea_orm::DbErr> {
        // Insert or replace behaviour: if exists, update html and reset formatted
        if let Some(existing) = CachedArticles::find()
            .filter(cached_articles::Column::Word.eq(word.to_string()))
            .filter(cached_articles::Column::LangDirection.eq(dir.to_string()))
            .one(&self.db)
            .await?
        {
            let mut am: ActiveModel = existing.into();
            am.html = Set(html.to_string());
            am.formatted = Set(None);
            am.updated_at = Set(Utc::now().naive_utc());
            am.update(&self.db).await?;
            Ok(())
        } else {
            let now = Utc::now().naive_utc();
            let mut am: ActiveModel = Default::default();
            am.word = Set(word.to_string());
            am.lang_direction = Set(dir.to_string());
            am.html = Set(html.to_string());
            am.formatted = Set(None);
            am.created_at = Set(now);
            am.updated_at = Set(now);
            am.insert(&self.db).await?;
            Ok(())
        }
    }

    pub async fn update_formatted(
        &self,
        word: &str,
        dir: &str,
        formatted: &str,
    ) -> Result<(), sea_orm::DbErr> {
        if let Some(existing) = CachedArticles::find()
            .filter(cached_articles::Column::Word.eq(word.to_string()))
            .filter(cached_articles::Column::LangDirection.eq(dir.to_string()))
            .one(&self.db)
            .await?
        {
            let mut am: ActiveModel = existing.into();
            am.formatted = Set(Some(formatted.to_string()));
            am.updated_at = Set(Utc::now().naive_utc());
            am.update(&self.db).await?;
            Ok(())
        } else {
            // If not exists, create a row with formatted (html empty)
            let now = Utc::now().naive_utc();
            let mut am: ActiveModel = Default::default();
            am.word = Set(word.to_string());
            am.lang_direction = Set(dir.to_string());
            am.html = Set("".to_string());
            am.formatted = Set(Some(formatted.to_string()));
            am.created_at = Set(now);
            am.updated_at = Set(now);
            am.insert(&self.db).await?;
            Ok(())
        }
    }
}
