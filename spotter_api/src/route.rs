use crate::handle::*;
use axum::{Router, routing::get}; 
use sqlx::PgPool;

pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/top/artists", get(get_top_artists))
        .route("/top/albums", get(get_top_albums))
        .route("/top/tracks", get(get_top_tracks))
        .route("/history", get(get_play_history))
        .route("/recents", get(get_recents))
        .route("/stats", get(get_stats))
        .with_state(pool)
}
