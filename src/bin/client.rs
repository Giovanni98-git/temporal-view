use temporal_client::{WorkflowClientTrait, WorkflowOptions};
use executor::helpers::client::get_client;
use temporal_sdk_core_protos::coresdk::AsJsonPayloadExt;
use tokio::time::{sleep, Duration};
use log::{info, error};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Charger les variables d'environnement et initialiser le logger
    dotenv::dotenv().ok();
    env_logger::init();

    // Obtenir le client Temporal
    let client = get_client().await?;

    // Générer un ID unique pour le workflow
    let workflow_id = format!("repeat-workflow-{}", Uuid::new_v4());
    info!("🚀 Starting workflow with ID: {}", workflow_id);

    // Démarrer le workflow
    let handle = client
        .start_workflow(
            vec!["".as_json_payload()?.into()],
            "repeat-task-queue".to_string(),
            workflow_id.clone(),
            "repeat_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    let run_id = handle.run_id;
    
    info!("✅ Workflow started");
    info!("🔎 Workflow ID: {}", workflow_id);
    info!("🔁 Run ID: {}", run_id);

    // Surveiller le statut du workflow
    info!("📡 Monitoring workflow status...");
    loop {
        // Interroger l'état du workflow
        let description = client
            .describe_workflow_execution(workflow_id.clone(), Some(run_id.clone()))
            .await?;

        // Récupérer le statut
        let status = description
            .workflow_execution_info
            .map(|info| info.status())
            .unwrap_or_default();

        // Afficher le statut
        match status {
            temporal_sdk_core_protos::temporal::api::enums::v1::WorkflowExecutionStatus::Running => {
                info!("⏳ Workflow is RUNNING");
            }
            temporal_sdk_core_protos::temporal::api::enums::v1::WorkflowExecutionStatus::Completed => {
                info!("🎉 Workflow COMPLETED");
                break; // Sortir de la boucle si le workflow est terminé
            }
            temporal_sdk_core_protos::temporal::api::enums::v1::WorkflowExecutionStatus::Failed => {
                error!("❌ Workflow FAILED");
                break;
            }
            temporal_sdk_core_protos::temporal::api::enums::v1::WorkflowExecutionStatus::Canceled => {
                info!("🛑 Workflow CANCELED");
                break;
            }
            temporal_sdk_core_protos::temporal::api::enums::v1::WorkflowExecutionStatus::Terminated => {
                info!("⚰️ Workflow TERMINATED");
                break;
            }
            temporal_sdk_core_protos::temporal::api::enums::v1::WorkflowExecutionStatus::ContinuedAsNew => {
                info!("🔄 Workflow CONTINUED AS NEW");
                break;
            }
            temporal_sdk_core_protos::temporal::api::enums::v1::WorkflowExecutionStatus::TimedOut => {
                error!("⏰ Workflow TIMED OUT");
                break;
            }
            _ => {
                info!("❓ Unknown workflow status: {:?}", status);
            }
        }

        // Attendre avant la prochaine interrogation (par exemple, toutes les 2 secondes)
        sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}