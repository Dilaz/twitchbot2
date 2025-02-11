//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0-rc.5

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub username: String,
    pub is_bot: bool,
    pub last_seen_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(created_at)]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(updated_at)]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::channel_users::Entity")]
    ChannelUsers,
}

impl Related<super::channel_users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChannelUsers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
