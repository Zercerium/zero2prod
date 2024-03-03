use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Subscriptions::Table)
                    .add_column(ColumnDef::new(Subscriptions::Status).text())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Subscriptions::Table)
                    .drop_column(Subscriptions::Status)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Subscriptions {
    Table,
    Status,
}
