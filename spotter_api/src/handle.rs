use crate::queries::*;
use axum::{Json, extract::{Query, State}, http::StatusCode};
use chrono::{
    DateTime,
    Utc,
};
use sqlx::PgPool;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TopParams {
    dt_start: DateTime<Utc>,
    dt_end: DateTime<Utc>,
    limit: i32,
}

#[derive(Deserialize)]
pub struct HistoryParams {
    dt_start: DateTime<Utc>,
    dt_end: DateTime<Utc>,
}

pub async fn get_top_artists(State(pool): State<PgPool>, Query(params): Query<TopParams>)
    -> Result<Json<Vec<ListArtist>>, StatusCode> {
        match query_top_artists(&pool, params.dt_start, params.dt_end, params.limit).await {
            Ok(Some(vector)) => Ok(Json(vector)),
            Ok(None) => Ok(Json(vec![])),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
}

pub async fn get_top_albums(State(pool): State<PgPool>, Query(params): Query<TopParams>)
    -> Result<Json<Vec<ListAlbum>>, StatusCode> {
        match query_top_albums(&pool, params.dt_start, params.dt_end, params.limit).await {
            Ok(Some(vector)) => Ok(Json(vector)),
            Ok(None) => Ok(Json(vec![])),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
}
pub async fn get_top_tracks(State(pool): State<PgPool>, Query(params): Query<TopParams>)
    -> Result<Json<Vec<ListTrack>>, StatusCode> {
        match query_top_tracks(&pool, params.dt_start, params.dt_end, params.limit).await {
            Ok(Some(vector)) => Ok(Json(vector)),
            Ok(None) => Ok(Json(vec![])),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
}
pub async fn get_play_history(State(pool): State<PgPool>, Query(params): Query<HistoryParams>)
    -> Result<Json<Vec<ListPlay>>, StatusCode> {
        match query_play_history(&pool, params.dt_start, params.dt_end).await {
            Ok(Some(vector)) => Ok(Json(vector)),
            Ok(None) => Ok(Json(vec![])),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
}
