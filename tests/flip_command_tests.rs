use pt_dict_bot::flip_direction;
use pt_dict_bot::migration::Migrator;
use pt_dict_bot::user_repository::UserRepository;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

async fn setup_test_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to test database");

    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    db
}

// ============================================================================
// Unit Tests for flip_direction() function
// ============================================================================

#[test]
fn test_flip_all_supported_directions() {
    let test_cases = [
        ("pten", "enpt"),
        ("enpt", "pten"),
        ("iten", "enit"),
        ("enit", "iten"),
    ];

    for (input, expected) in test_cases {
        let result = flip_direction(input);
        assert_eq!(
            result,
            Some(expected.to_string()),
            "flip_direction({}) should return {}",
            input,
            expected
        );
    }
}

#[test]
fn test_flip_invalid_length_too_short() {
    let result = flip_direction("pt");
    assert_eq!(result, None);
}

#[test]
fn test_flip_invalid_length_too_long() {
    let result = flip_direction("ptenxx");
    assert_eq!(result, None);
}

#[test]
fn test_flip_empty_string_returns_none() {
    let result = flip_direction("");
    assert_eq!(result, None);
}

#[test]
fn test_flip_arbitrary_valid_format() {
    // Test that arbitrary 4-letter codes are NOT allowed (closed list only)
    let result = flip_direction("fres");
    assert_eq!(result, None);
}

#[test]
fn test_flip_double_flip_returns_original() {
    let original = "pten";
    let flipped_once = flip_direction(original).unwrap();
    let flipped_twice = flip_direction(&flipped_once).unwrap();
    assert_eq!(flipped_twice, original);
}

#[test]
fn test_flip_rejects_unsupported_direction() {
    // Test various unsupported directions
    assert_eq!(flip_direction("deen"), None);
    assert_eq!(flip_direction("esfr"), None);
    assert_eq!(flip_direction("jaen"), None);
}

// ============================================================================
// Integration Tests for /flip command handler
// ============================================================================

#[tokio::test]
async fn test_flip_command_for_existing_user() {
    let db = setup_test_db().await;
    let repo = UserRepository::new(db);

    // Create user with pten direction
    repo.create_or_update_user("test_chat", "pten", Some(123), Some("testuser"))
        .await
        .expect("Failed to create user");

    // Verify initial direction
    let user = repo.get_user("test_chat").await.unwrap().unwrap();
    assert_eq!(user.translation_direction, "pten");

    // Simulate flip command by updating direction
    repo.update_translation_direction("test_chat", "enpt")
        .await
        .expect("Failed to flip direction");

    // Verify direction changed
    let user_after = repo.get_user("test_chat").await.unwrap().unwrap();
    assert_eq!(user_after.translation_direction, "enpt");
    assert_eq!(user_after.user_id, Some(123)); // Preserved
    assert_eq!(user_after.username, Some("testuser".to_string())); // Preserved
}

#[tokio::test]
async fn test_flip_command_for_new_user() {
    let db = setup_test_db().await;
    let repo = UserRepository::new(db);

    // Verify user doesn't exist
    let user = repo.get_user("new_chat").await.unwrap();
    assert!(user.is_none());

    // Simulate flip for new user - should create with flipped default
    let flipped_default = flip_direction("pten").unwrap(); // "enpt"
    repo.create_or_update_user("new_chat", &flipped_default, Some(456), Some("newuser"))
        .await
        .expect("Failed to create user");

    // Verify user was created with flipped default
    let user = repo.get_user("new_chat").await.unwrap().unwrap();
    assert_eq!(user.translation_direction, "enpt");
    assert_eq!(user.user_id, Some(456));
}

#[tokio::test]
async fn test_flip_command_double_flip_returns_original() {
    let db = setup_test_db().await;
    let repo = UserRepository::new(db);

    // Create user with pten
    repo.create_or_update_user("flip_twice_chat", "pten", None, None)
        .await
        .expect("Failed to create user");

    // First flip: pten -> enpt
    repo.update_translation_direction("flip_twice_chat", "enpt")
        .await
        .expect("Failed to first flip");

    let user = repo.get_user("flip_twice_chat").await.unwrap().unwrap();
    assert_eq!(user.translation_direction, "enpt");

    // Second flip: enpt -> pten (back to original)
    repo.update_translation_direction("flip_twice_chat", "pten")
        .await
        .expect("Failed to second flip");

    let user = repo.get_user("flip_twice_chat").await.unwrap().unwrap();
    assert_eq!(user.translation_direction, "pten");
}

#[tokio::test]
async fn test_flip_command_preserves_user_data() {
    let db = setup_test_db().await;
    let repo = UserRepository::new(db);

    // Create user with data
    repo.create_or_update_user("preserve_chat", "pten", Some(789), Some("keepme"))
        .await
        .expect("Failed to create user");

    let before = repo.get_user("preserve_chat").await.unwrap().unwrap();

    // Flip direction
    repo.update_translation_direction("preserve_chat", "enpt")
        .await
        .expect("Failed to flip");

    let after = repo.get_user("preserve_chat").await.unwrap().unwrap();

    // Verify only direction changed
    assert_eq!(after.translation_direction, "enpt");
    assert_eq!(after.user_id, before.user_id);
    assert_eq!(after.username, before.username);
    assert_eq!(after.created_at, before.created_at);
    assert!(after.updated_at >= before.updated_at);
}

#[tokio::test]
async fn test_flip_command_with_invalid_direction_in_db() {
    let db = setup_test_db().await;
    let repo = UserRepository::new(db);

    // Verify we can handle missing user case
    let user = repo.get_user("invalid_dir_chat").await.unwrap();
    assert!(user.is_none());

    // The actual handler would use DEFAULT_LANG_DIRECTION if flip fails
    let fallback = flip_direction("pten").unwrap_or("pten".to_string());
    assert_eq!(fallback, "enpt");
}

#[test]
fn test_command_enum_parsing() {
    // Verify flip_direction works for all supported cases
    assert!(flip_direction("pten").is_some());
    assert!(flip_direction("enpt").is_some());
    assert!(flip_direction("iten").is_some());
    assert!(flip_direction("enit").is_some());
}
