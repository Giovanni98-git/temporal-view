use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_executions_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Execution::Table)
                    .col(
                        ColumnDef::new(Execution::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Execution::WorkflowId).string().not_null())
                    .col(ColumnDef::new(Execution::RunId).string().not_null())
                    .col(ColumnDef::new(Execution::Status).string().not_null())
                    .col(
                        ColumnDef::new(Execution::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Execution::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Execution {
    #[iden = "executions"]
    Table,
    Id,
    WorkflowId,
    RunId,
    Status,
    CreatedAt,
}