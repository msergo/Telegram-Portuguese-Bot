use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251011_000001_create_users_table::Migration),
            Box::new(m20251011_000002_create_cached_articles_table::Migration),
        ]
    }
}

pub mod m20251011_000001_create_users_table;
pub mod m20251011_000002_create_cached_articles_table;
