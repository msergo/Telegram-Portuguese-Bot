use crate::entities::users::{ActiveModel, Entity as Users, Model};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

#[derive(Clone)]
pub struct UserRepository {
    pub db: DatabaseConnection,
}

impl UserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_user(&self, chat_id: &str) -> Result<Option<Model>, sea_orm::DbErr> {
        Users::find_by_id(chat_id.to_string()).one(&self.db).await
    }

    pub async fn create_or_update_user(
        &self,
        chat_id: &str,
        translation_direction: &str,
        user_id: Option<i64>,
        username: Option<&str>,
    ) -> Result<Model, sea_orm::DbErr> {
        let now = Utc::now().naive_utc();

        let user = ActiveModel {
            chat_id: Set(chat_id.to_string()),
            translation_direction: Set(translation_direction.to_string()),
            user_id: Set(user_id),
            username: Set(username.map(|s| s.to_string())),
            created_at: Set(now),
            updated_at: Set(now),
        };

        user.insert(&self.db).await
    }

    pub async fn update_translation_direction(
        &self,
        chat_id: &str,
        direction: &str,
    ) -> Result<Model, sea_orm::DbErr> {
        let mut user: ActiveModel = Users::find_by_id(chat_id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("User not found".to_string()))?
            .into();

        user.translation_direction = Set(direction.to_string());
        user.updated_at = Set(Utc::now().naive_utc());
        user.update(&self.db).await
    }
}
