use pt_dict_bot::constants::LANG_PT_EN;
use pt_dict_bot::migration::Migrator;
use pt_dict_bot::user_repository::UserRepository;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

async fn setup_test_db() -> DatabaseConnection {
    // Create in-memory SQLite database for testing
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    db
}

#[tokio::test]
async fn test_create_and_get_user() {
    let db = setup_test_db().await;
    let repo = UserRepository::new(db);

    // Create user
    let user = repo
        .create_or_update_user("123", LANG_PT_EN, Some(456789), Some("johndoe"))
        .await
        .expect("Failed to create user");

    assert_eq!(user.chat_id, "123");
    assert_eq!(user.translation_direction, LANG_PT_EN);
    assert_eq!(user.user_id, Some(456789));
    assert_eq!(user.username, Some("johndoe".to_string()));

    // Get user
    let retrieved = repo
        .get_user("123")
        .await
        .expect("Failed to get user")
        .expect("User should exist");

    assert_eq!(retrieved.chat_id, "123");
    assert_eq!(retrieved.translation_direction, LANG_PT_EN);
}

#[tokio::test]
async fn test_update_translation_direction() {
    let db = setup_test_db().await;
    let repo = UserRepository::new(db);

    // Create user
    repo.create_or_update_user("456", "pten", None, None)
        .await
        .expect("Failed to create user");

    // Update direction
    let updated = repo
        .update_translation_direction("456", "iten")
        .await
        .expect("Failed to update direction");

    assert_eq!(updated.translation_direction, "iten");
}

#[tokio::test]
async fn test_get_nonexistent_user() {
    let db = setup_test_db().await;
    let repo = UserRepository::new(db);

    let result = repo
        .get_user("nonexistent")
        .await
        .expect("Query should succeed");

    assert!(result.is_none());
}
