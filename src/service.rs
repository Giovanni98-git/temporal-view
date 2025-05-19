use anyhow::Error;
use sea_orm::{ ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set};
use uuid::Uuid;
use crate::{executions::{self, Entity as Execution}, helpers::client::get_client};
use temporal_client::{WorkflowClientTrait, WorkflowOptions};
use temporal_sdk_core_protos::coresdk::AsJsonPayloadExt;

// Structure pour les données d'entrée lors de la création ou mise à jour
#[derive(Debug)]
pub struct ExecutionInput {
    pub id: Uuid,
    pub workflow_id: String,
    pub run_id: String,
    pub status: String,
}


// Créer une nouvelle exécution
pub async fn create_execution(db: &DatabaseConnection, input: ExecutionInput) -> Result<executions::Model, DbErr> {
    let execution = executions::ActiveModel {
        id: Set(input.id),
        workflow_id: Set(input.workflow_id),
        run_id: Set(input.run_id),
        status: Set(input.status),
    };
    let result = execution.insert(db).await?;
    Ok(result)
}

// Récupérer une exécution par son ID
pub async fn get_execution_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<executions::Model>, DbErr> {
    Execution::find_by_id(id).one(db).await
}

// Mettre à jour une exécution existante
pub async fn update_execution(db: &DatabaseConnection, id: Uuid, input: ExecutionInput) -> Result<executions::Model, DbErr> {
    let execution: Option<executions::Model> = Execution::find_by_id(id).one(db).await?;
    if execution.is_none() {
        return Err(DbErr::RecordNotFound(format!("Execution with id {} not found", id)));
    }

    let mut execution: executions::ActiveModel = execution.unwrap().into();
    execution.workflow_id = Set(input.workflow_id);
    execution.run_id = Set(input.run_id);
    execution.status = Set(input.status);
    let result = execution.update(db).await?;
    Ok(result)
}

// Supprimer une exécution par son ID
pub async fn delete_execution(db: &DatabaseConnection, id: Uuid) -> Result<u64, DbErr> {
    let result = Execution::delete_by_id(id).exec(db).await?;
    Ok(result.rows_affected)
}

// Lister toutes les exécutions
pub async fn list_executions(db: &DatabaseConnection) -> Result<Vec<executions::Model>, DbErr> {
    Execution::find().all(db).await
}

pub async fn list_incomplete_executions(db: &DatabaseConnection) -> Result<Vec<executions::Model>, DbErr> {
    Execution::find()
        .filter(executions::Column::Status.eq("RUNNING"))
        .all(db)
        .await
}

// initier la tache 
pub async fn init_workflow() -> Result<(String, String), Error> {
    // Obtenir le client et déballer le Result
    let client = get_client().await?;

    // Générer un ID unique pour le workflow
    let workflow_id = format!("wf-{}", Uuid::new_v4());

    // Démarrer le workflow
    let handle = client
        .start_workflow(
            vec!["".as_json_payload().expect("Failed to create payload")],
            "repeat-task-queue".to_string(),
            workflow_id.clone(),
            "repeat_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    Ok((workflow_id, handle.run_id))
}

