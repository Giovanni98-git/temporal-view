use std::env;

use actix_web::{delete, get, post, web, App, HttpResponse, HttpServer, Responder};
use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;
use uuid::Uuid;
use crate::database::init_db;
use crate::service::{create_execution, delete_execution, get_execution_by_id, init_workflow, list_executions, ExecutionInput};

// Créer une nouvelle exécution
#[post("/executions")]
async fn add_execution(db: web::Data<DatabaseConnection>) -> impl Responder {

    match init_workflow().await {
        Ok((workflow_id, run_id)) => {
            let execution_input = ExecutionInput {
                id: Uuid::new_v4(),
                workflow_id,
                run_id,
                status: "RUNNING".to_string(),
            };

            match create_execution(&db, execution_input).await {
                Ok(execution) => HttpResponse::Ok().json(execution),
                Err(e) => {
                    HttpResponse::InternalServerError().body(format!("Échec de la création de l'exécution:{}",e))
                }
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Échec du démarrage du workflow: {}", e))
        }
    }
}

// Récupérer une exécution par ID
#[get("/executions/{id}")]
async fn get_execution(id: web::Path<Uuid>, db: web::Data<DatabaseConnection>) -> impl Responder {
    match get_execution_by_id(&db, id.into_inner()).await {
        Ok(Some(execution)) => HttpResponse::Ok().json(execution),
        Ok(None) => HttpResponse::NotFound().body("Exécution non trouvée"),
        Err(_) => HttpResponse::InternalServerError().body("Échec de la récupération de l'exécution"),
    }
}

// Supprimer une exécution
#[delete("/executions/{id}")]
async fn delete_execution_endpoint(id: web::Path<Uuid>, db: web::Data<DatabaseConnection>) -> impl Responder {
    match delete_execution(&db, id.into_inner()).await {
        Ok(rows_affected) if rows_affected > 0 => HttpResponse::Ok().body("Exécution supprimée"),
        Ok(_) => HttpResponse::NotFound().body("Exécution non trouvée"),
        Err(_) => HttpResponse::InternalServerError().body("Échec de la suppression de l'exécution"),
    }
}

// Lister toutes les exécutions
#[get("/executions")]
async fn list_execution(db: web::Data<DatabaseConnection>) -> impl Responder {
    match list_executions(&db).await {
        Ok(executions) => HttpResponse::Ok().json(executions),
        Err(_) => HttpResponse::InternalServerError().body("Échec de la récupération des exécutions"),
    }
}

// Lancer le serveur
pub async fn run_server() -> std::io::Result<()> {
    let db = init_db().await;
    Migrator::up(&db, None).await.expect("Échec de l'application des migrations");
    // Read the Temporal server address from environment variable, with fallback
    let server_url = env::var("SERVER_URL")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    println!("serveur démarré sur http://{}", &server_url);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .service(add_execution)
            .service(get_execution)
            .service(delete_execution_endpoint)
            .service(list_execution)
    })
    .bind(&server_url)?
    .run()
    .await
}