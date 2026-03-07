mod queries;
mod handle;
mod route;
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&url).await.unwrap();

    let app = route::create_router(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4005").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}
