use executor::{controller, database::init_db, workers};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialiser le logger
    env_logger::init();
    dotenv::dotenv().ok();
    println!("🚀 Starting application");

    println!("🔧 Starting Temporal worker...");
    println!("🌐 Starting Actix Web server...");

    // Lancer worker et serveur en parallèle, sans tokio::spawn
    let (worker_result, server_result,()) = tokio::join!(
        async {
            if let Err(e) = workers::start_worker().await {
                log::error!("❌ Failed to start worker: {:?}", e);
                Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            } else {
                println!("✅ Worker started successfully");
                Ok(())
            }
        },
        async {
            controller::run_server().await
        },
        async {
            let db = std::sync::Arc::new(init_db().await);

            // 🔁 Lancer le scheduler en tâche de fond
            crate::workers::start_execution_status_scheduler(db.clone()).await;
        }
    );

    // Propager les erreurs si nécessaire
    worker_result?;
    server_result?;

    Ok(())
}
