use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .create_table(
                Table::create()
                    .table(BannedWord::Table)
                    .if_not_exists()
                    .col(pk_auto(BannedWord::Id))
                    .col(string(BannedWord::Word).not_null())
                    .col(boolean(BannedWord::IsRegex).not_null())
                    .col(integer_null(BannedWord::ChannelId))
                    .col(timestamp_with_time_zone(BannedWord::CreatedAt).not_null().default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone(BannedWord::UpdatedAt).not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await;

            manager.create_foreign_key(ForeignKeyCreateStatement::new()
                .from_tbl(BannedWord::Table)
                .from_col(BannedWord::ChannelId)
                .to_tbl(Channel::Table)
                .to_col(Channel::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Restrict)
                .to_owned()
            ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BannedWord::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum BannedWord {
    #[sea_orm(iden = "banned_words")]
    Table,
    Id,
    Word,
    IsRegex,
    ChannelId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Channel {
    #[sea_orm(iden = "channels")]
    Table,
    Id,
}