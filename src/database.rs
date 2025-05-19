use sea_orm::{Database, DatabaseConnection};

pub async fn init_db() -> DatabaseConnection {
    let db_url = "sqlite:db.sqlite?mode=rwc";
    Database::connect(db_url).await.expect("Échec de la connexion à la base de données")
}