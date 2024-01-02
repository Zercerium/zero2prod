pub use sea_orm_migration::prelude::*;

mod m20240102_131000_create_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20240102_131000_create_table::Migration)]
    }
}
