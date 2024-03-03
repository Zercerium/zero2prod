use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SubscriptionsTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SubscriptionsTokens::SubscriptionsToken)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionsTokens::SubscriberId)
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(SubscriptionsTokens::Table)
                            .from_col(SubscriptionsTokens::SubscriberId)
                            .to_tbl(Subscriptions::Table)
                            .to_col(Subscriptions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SubscriptionsTokens::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SubscriptionsTokens {
    Table,
    SubscriptionsToken,
    SubscriberId,
}

#[derive(DeriveIden)]
pub enum Subscriptions {
    Table,
    Id,
}
