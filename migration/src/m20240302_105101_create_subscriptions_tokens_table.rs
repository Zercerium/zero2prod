use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SubscriptionTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SubscriptionTokens::SubscriptionToken)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionTokens::SubscriberId)
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from_tbl(SubscriptionTokens::Table)
                            .from_col(SubscriptionTokens::SubscriberId)
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
            .drop_table(Table::drop().table(SubscriptionTokens::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SubscriptionTokens {
    Table,
    SubscriptionToken,
    SubscriberId,
}

#[derive(DeriveIden)]
pub enum Subscriptions {
    Table,
    Id,
}
