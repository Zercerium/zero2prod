use sea_orm_migration::{prelude::*, sea_orm::prelude::Uuid};

// quite unsure about adding the entity crate here, in this case something could brake if the entities are getting updated
// but the migration step stays on the same place where the creation the entity model is still valid
// other possibility is to always move the seed migration to the end of all migrations, where the entities are always up date with
// the current schema

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let insert = Query::insert()
            .into_table(Users::Table)
            .columns([Users::UserId, Users::Username, Users::PasswordHash])
            .values_panic([
                Uuid::parse_str("ddf8994f-d522-4659-8d02-c1d479057be6")
                    .unwrap()
                    .into(),
                "admin".into(),
                "$argon2id$v=19$m=15000,t=2,p=1$OEx/rcq+3ts//'
                'WUDzGNl2g$Am8UFBA4w5NJEmAtquGvBmAlu92q/VQcaoL5AyJPfc8"
                    .into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let delete = Query::delete()
            .from_table(Users::Table)
            .and_where(Expr::col(Users::UserId).eq("ddf8994f-d522-4659-8d02-c1d479057be6"))
            .to_owned();

        manager.exec_stmt(delete).await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    UserId,
    Username,
    PasswordHash,
}
