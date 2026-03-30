mod config;
use axum::{
    routing::{get, post},
    http::{HeaderValue, Method},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone)]
pub struct AppData {
    pool: sqlx::PgPool,
    reqwest: reqwest::Client
}

#[tokio::main]
async fn main() {
    env_logger::builder().init();

    let reqwest = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .expect("Could not initialize reqwest client");

    let cfg = &*config::CONFIG;

    let pool = PgPoolOptions::new()
        .max_connections(cfg.max_db_connections)
        .connect(&cfg.postgres_url)
        .await
        .expect("Could not initialize connection");

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::POST]);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/signin", post(signin))
        .with_state(AppData { pool, reqwest })
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn signin() -> &'static str {
    println!("Hello World");
    "Hello World"
}
