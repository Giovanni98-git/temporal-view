use std::{sync::Arc, time::Duration};
use actix_web::web::Data;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use temporal_client::WorkflowClientTrait;
use temporal_sdk::Worker;
use temporal_sdk_core::{init_worker, CoreRuntime};
use temporal_sdk_core_api::{
    telemetry::TelemetryOptionsBuilder,
    worker::{WorkerConfigBuilder, WorkerVersioningStrategy},
};
use log::info;
use temporal_sdk_core_protos::temporal::api::enums::v1::WorkflowExecutionStatus;
use tokio::time::interval;

use crate::{helpers::client::get_client, service::{list_incomplete_executions, update_execution, ExecutionInput}, workflows::{repeat_activity, repeat_workflow}};

// Structure pour la tâche de mise à jour des statuts
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateExecutionStatusJob;

pub async fn start_worker() -> Result<(), Box<dyn std::error::Error>> {
    let client = crate::helpers::client::get_client().await?;

    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;
    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("repeat-task-queue")
        .versioning_strategy(WorkerVersioningStrategy::default())
        .build()?;

    let core_worker = init_worker(&runtime, worker_config, client)?;
    let mut worker = Worker::new_from_core(Arc::new(core_worker), "repeat-task-queue");

    worker.register_activity("repeat_activity", repeat_activity);
    worker.register_wf("repeat_workflow", repeat_workflow);

    info!("🎧 Worker running and waiting for tasks...");
    worker.run().await.map_err(|e| {
        log::error!("Worker failed: {:?}", e);
        e
    })?;

    Ok(())
}

// Convertit un statut Temporal en chaîne
fn workflow_status_to_string(status: WorkflowExecutionStatus) -> &'static str {
    match status {
        WorkflowExecutionStatus::Running => "RUNNING",
        WorkflowExecutionStatus::Completed => "COMPLETE",
        WorkflowExecutionStatus::Failed => "FAILED",
        WorkflowExecutionStatus::Canceled => "CANCELED",
        WorkflowExecutionStatus::Terminated => "TERMINATED",
        WorkflowExecutionStatus::ContinuedAsNew => "CONTINUED_AS_NEW",
        WorkflowExecutionStatus::TimedOut => "TIMED_OUT",
        _ => "UNKNOWN",
    }
}

// Worker qui met à jour le statut des exécutions
async fn update_execution_status_worker(
    _job: UpdateExecutionStatusJob,
    db: Data<Arc<DatabaseConnection>>,
) -> Result<(), anyhow::Error> {
    log::info!("Starting execution status update job");

    let client = get_client().await?;
    let executions = list_incomplete_executions(&db).await?;

    for exec in executions {
        match client.describe_workflow_execution(exec.workflow_id.clone(), Some(exec.run_id.clone())).await {
            Ok(description) => {
                let status = description
                    .workflow_execution_info
                    .as_ref()
                    .map(|info| info.status())
                    .unwrap_or(WorkflowExecutionStatus::Unspecified);

                let input = ExecutionInput {
                    id: exec.id,
                    workflow_id: exec.workflow_id,
                    run_id: exec.run_id,
                    status: workflow_status_to_string(status).to_string(),
                };

                match update_execution(&db, exec.id, input).await {
                    Ok(updated_exec) => log::info!("Updated execution {} to status {}", exec.id, updated_exec.status),
                    Err(err) => log::error!("Failed to update execution {}: {}", exec.id, err),
                }
            }
            Err(err) => {
                log::error!("Failed to describe workflow {}: {}", exec.workflow_id, err);
                continue;
            }
        }
    }

    Ok(())
}

pub async fn start_execution_status_scheduler(db: Arc<DatabaseConnection>) {
    let job = UpdateExecutionStatusJob;
    let db_data = Data::new(db);

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            if let Err(err) = update_execution_status_worker(job.clone(), db_data.clone()).await {
                log::error!("❌ Failed to run update_execution_status_worker: {:?}", err);
            } else {
                log::info!("✅ update_execution_status_worker ran successfully");
            }
        }
    });
}
