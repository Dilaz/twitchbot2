pub use sea_orm_migration::prelude::*;
mod m20220101_000001_create_channels_table;
mod m20220101_000002_create_banned_words_table;
mod m20241110_180847_create_users_table;
mod m20241111_195113_create_channel_users_table;
mod m20241111_195118_create_urls_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_channels_table::Migration),
            Box::new(m20220101_000002_create_banned_words_table::Migration),
            Box::new(m20241110_180847_create_users_table::Migration),
            Box::new(m20241111_195113_create_channel_users_table::Migration),
            Box::new(m20241111_195118_create_urls_table::Migration),
        ]
    }
}
