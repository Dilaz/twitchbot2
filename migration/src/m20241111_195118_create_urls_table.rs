use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Url::Table)
                    .if_not_exists()
                    .col(pk_auto(Url::Id))
                    .col(string(Url::Url).unique_key())
                    .col(boolean(Url::Spam).not_null().default(false))
                    .col(timestamp_with_time_zone(Url::CreatedAt).not_null().default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone(Url::UpdatedAt).not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Url::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Url {
    #[sea_orm(iden = "urls")]
    Table,
    Id,
    Url,
    Spam,
    CreatedAt,
    UpdatedAt,
}
