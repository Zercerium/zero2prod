use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let insert = Query::update()
            .table(Subscriptions::Table)
            .value(Subscriptions::Status, "confirmed")
            .and_where(Expr::col(Subscriptions::Status).is_null())
            .to_owned();
        manager.exec_stmt(insert).await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Subscriptions::Table)
                    .modify_column(ColumnDef::new(Subscriptions::Status).not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Subscriptions::Table)
                    .modify_column(ColumnDef::new(Subscriptions::Status).null())
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
