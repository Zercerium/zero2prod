pub use sea_orm_migration::prelude::*;

mod m20240102_131000_create_table;
mod m20240302_100319_add_status_to_subscriptions;
mod m20240302_102407_add_make_status_not_null_in_subscriptions;
mod m20240302_105101_create_subscriptions_tokens_table;
mod m20240307_150649_create_users_table;
mod m20240309_120745_seed_user;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240102_131000_create_table::Migration),
            Box::new(m20240302_100319_add_status_to_subscriptions::Migration),
            Box::new(m20240302_102407_add_make_status_not_null_in_subscriptions::Migration),
            Box::new(m20240302_105101_create_subscriptions_tokens_table::Migration),
            Box::new(m20240307_150649_create_users_table::Migration),
            Box::new(m20240309_120745_seed_user::Migration),
        ]
    }
}
