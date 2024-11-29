use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Channel::Table)
                    .if_not_exists()
                    .col(pk_auto(Channel::Id))
                    .col(string(Channel::Name).not_null().unique_key())
                    .col(json(Channel::Settings).not_null().default("{}"))
                    .col(timestamp_with_time_zone(Channel::CreatedAt).not_null().default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone(Channel::UpdatedAt).not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

            let insert = Query::insert()
            .into_table(Channel::Table)
            .columns([Channel::Name, Channel::CreatedAt, Channel::UpdatedAt])
            .values_panic([
                SimpleExpr::Value(Value::String(Some(Box::new("Dilaz".to_string())))),
                SimpleExpr::Value(Value::ChronoDateTimeUtc(Some(Box::new(chrono::Utc::now())))),
                SimpleExpr::Value(Value::ChronoDateTimeUtc(Some(Box::new(chrono::Utc::now())))),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Channel::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Channel {
    #[sea_orm(iden = "channels")]
    Table,
    Id,
    Name,
    Settings,
    CreatedAt,
    UpdatedAt,
}
