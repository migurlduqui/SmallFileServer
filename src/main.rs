mod models;
mod routes;
use tokio::net::{TcpListener};
use tower_http::cors::CorsLayer;
use axum::Router;
use axum::routing::{get, post};
use tower_http::services::ServeDir;
use routes::{download,upload};


#[tokio::main]
async fn main() {
    // Initialize logging
    
    // Build the routerx
    let app = Router::new()
        .route("/api/upload", post(upload))
        .route("/api/download/{filename}", get(download))
        .route("/api/list", get(routes::list))
        .layer(CorsLayer::permissive());
    
    // Start the server
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}