use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CachedArticles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CachedArticles::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(CachedArticles::Word).string().not_null())
                    .col(
                        ColumnDef::new(CachedArticles::LangDirection)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CachedArticles::Html).string().not_null())
                    .col(ColumnDef::new(CachedArticles::Formatted).string().null())
                    .col(
                        ColumnDef::new(CachedArticles::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(CachedArticles::UpdatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CachedArticles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CachedArticles {
    Table,
    Id,
    Word,
    LangDirection,
    Html,
    Formatted,
    CreatedAt,
    UpdatedAt,
}
