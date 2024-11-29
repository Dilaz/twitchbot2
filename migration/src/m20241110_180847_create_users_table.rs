use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_auto(User::Id))
                    .col(string(User::Username).unique_key())
                    .col(boolean(User::IsBot).default(false))
                    .col(timestamp_with_time_zone_null(User::LastSeenAt))
                    .col(timestamp_with_time_zone(User::CreatedAt).not_null().default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone(User::UpdatedAt).not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    #[sea_orm(iden = "users")]
    Table,
    Id,
    Username,
    IsBot,
    LastSeenAt,
    CreatedAt,
    UpdatedAt,
}
