use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChannelUser::Table)
                    .if_not_exists()
                    .col(pk_auto(ChannelUser::Id))
                    .col(timestamp_with_time_zone_null(ChannelUser::LastSeenAt))
                    .col(unsigned(ChannelUser::ChannelId).not_null())
                    .col(unsigned(ChannelUser::UserId).not_null())
                    .col(date_time(ChannelUser::CreatedAt).not_null().default(Expr::current_timestamp()))
                    .col(date_time(ChannelUser::UpdatedAt).not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;
        
        manager.create_foreign_key(
            ForeignKey::create()
                .from(ChannelUser::Table, ChannelUser::ChannelId)
                .to(Channel::Table, Channel::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .from(ChannelUser::Table, ChannelUser::UserId)
                .to(User::Table, User::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChannelUser::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ChannelUser {
    #[sea_orm(iden = "channel_users")]
    Table,
    Id,
    LastSeenAt,
    ChannelId,
    UserId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Channel {
    #[sea_orm(iden = "channels")]
    Table,
    Id,
}

#[derive(DeriveIden)]
enum User {
    #[sea_orm(iden = "users")]
    Table,
    Id,
}
