use axum::{
    routing::{get, post},
    http::{HeaderValue, Method},
    Router,
};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::POST]);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/signin", post(signin))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn signin() -> &'static str {
    println!("Hello World");
    "Hello World"
}
