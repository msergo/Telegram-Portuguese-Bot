use pt_dict_bot::entities::cached_articles::Entity as CachedArticles;
use pt_dict_bot::entities::users::Entity as Users;
use pt_dict_bot::migration::Migrator;
use sea_orm::Database;
use sea_orm::EntityTrait;
use sea_orm_migration::MigratorTrait;

#[tokio::test]
async fn test_migrations_create_tables() {
    // Use file-based DB to let migrations run as in production
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    // After migrations, the entities should be queryable (return Ok(None) if empty)
    let users = Users::find().one(&db).await.expect("Users query failed");
    let cached = CachedArticles::find()
        .one(&db)
        .await
        .expect("CachedArticles query failed");

    // We don't assert on data; existence of table is proven by successful query
    assert!(users.is_none());
    assert!(cached.is_none());
}
