use pt_dict_bot::constants::{DEFAULT_LANG_DIRECTION, LANG_IT_EN};
use pt_dict_bot::migration::Migrator;
use pt_dict_bot::user_repository::UserRepository;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;

#[tokio::test]
async fn test_user_config_integration() {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    let repo = UserRepository::new(db);

    // Test default fallback
    let default_direction = match repo.get_user("unknown_chat").await {
        Ok(Some(user)) => user.translation_direction,
        _ => DEFAULT_LANG_DIRECTION.to_string(),
    };
    assert_eq!(default_direction, DEFAULT_LANG_DIRECTION);

    // Test stored user config
    repo.create_or_update_user("test_chat", LANG_IT_EN, None, None)
        .await
        .expect("Failed to create user");

    let stored_direction = repo
        .get_user("test_chat")
        .await
        .expect("Failed to get user")
        .expect("User should exist")
        .translation_direction;

    assert_eq!(stored_direction, LANG_IT_EN);
}

#[tokio::test]
async fn test_chat_context_behavior() {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    let repo = UserRepository::new(db);

    // Simulate group chat scenario: group ID "group_123" has "iten" setting
    repo.create_or_update_user("group_123", LANG_IT_EN, None, None)
        .await
        .expect("Failed to create group config");

    // Any user mentioning bot in this group should get "iten" direction
    let group_direction = repo
        .get_user("group_123")
        .await
        .expect("Failed to get group config")
        .expect("Group config should exist")
        .translation_direction;

    assert_eq!(group_direction, LANG_IT_EN);

    // Individual users in the group don't have separate settings
    let individual_user_result = repo
        .get_user("user_456")
        .await
        .expect("Query should succeed");
    assert!(individual_user_result.is_none()); // No individual user config
}
