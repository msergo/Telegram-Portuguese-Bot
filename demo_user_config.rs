use pt_dict_bot::entities::users::Entity as Users;
use pt_dict_bot::migration::Migrator;
use pt_dict_bot::user_repository::UserRepository;
use sea_orm::{Database, EntityTrait};

#[tokio::main]
async fn main() {
    println!("=== SeaORM User Configuration Demo ===\n");

    // Create in-memory database for demo
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to database");

    // Run migrations
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    let repo = UserRepository::new(db.clone());

    println!("1. Creating user configurations...\n");

    // Simulate different chat scenarios
    let scenarios = vec![
        (
            "group_123456789",
            "pten",
            None,
            None,
            "Group chat with default Portuguese->English",
        ),
        (
            "private_987654321",
            "iten",
            Some(987654321),
            Some("john_doe"),
            "Private chat Italian->English",
        ),
        (
            "group_555666777",
            "enpt",
            None,
            None,
            "Group chat English->Portuguese",
        ),
    ];

    for (chat_id, direction, user_id, username, description) in scenarios {
        repo.create_or_update_user(chat_id, direction, user_id, username.as_deref())
            .await
            .expect("Failed to create user");

        println!("✓ Created: {} - {}", chat_id, description);
    }

    println!("\n2. Database contents:\n");

    // Show all users in the database
    let users = Users::find().all(&db).await.expect("Failed to fetch users");

    println!(
        "{:<20} {:<15} {:<12} {:<15} {:<25} {}",
        "chat_id", "direction", "user_id", "username", "created_at", "description"
    );
    println!("{}", "=".repeat(100));

    for user in users {
        let description = match user.chat_id.as_str() {
            "group_123456789" => "Group chat (Portuguese->English)",
            "private_987654321" => "Private chat (Italian->English)",
            "group_555666777" => "Group chat (English->Portuguese)",
            _ => "Unknown",
        };

        println!(
            "{:<20} {:<15} {:<12} {:<15} {:<25} {}",
            user.chat_id,
            user.translation_direction,
            user.user_id.map_or("NULL".to_string(), |id| id.to_string()),
            user.username.as_deref().unwrap_or("NULL"),
            user.created_at.format("%Y-%m-%d %H:%M:%S"),
            description
        );
    }

    println!("\n3. Testing user lookup...\n");

    // Test lookups
    let test_chat_ids = vec!["group_123456789", "private_987654321", "nonexistent_chat"];

    for chat_id in test_chat_ids {
        match repo.get_user(chat_id).await {
            Ok(Some(user)) => println!(
                "✓ Found {}: direction={}",
                chat_id, user.translation_direction
            ),
            Ok(None) => println!("✗ {}: not found (would use default 'pten')", chat_id),
            Err(e) => println!("✗ {}: error - {}", chat_id, e),
        }
    }

    println!("\n4. Testing direction updates...\n");

    // Test updating direction
    let chat_id = "group_123456789";
    println!("Updating {} from pten to iten...", chat_id);

    match repo.update_translation_direction(chat_id, "iten").await {
        Ok(updated_user) => println!(
            "✓ Updated: {} now uses {}",
            chat_id, updated_user.translation_direction
        ),
        Err(e) => println!("✗ Failed to update: {}", e),
    }

    println!("\n=== Demo Complete ===");
    println!("\nThe users table stores:");
    println!("- chat_id: Primary key (group ID for groups, user ID for private chats)");
    println!("- translation_direction: Current setting (pten, iten, enpt, enit)");
    println!("- user_id: Telegram user ID (NULL for group chats)");
    println!("- username: Telegram username (NULL for group chats)");
    println!("- created_at/updated_at: Timestamps");
}
