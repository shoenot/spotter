use crate::queries::*;
use axum::{Json, extract::{Query, State}, http::StatusCode};
use chrono::{DateTime, Datelike, FixedOffset, TimeZone};
use chrono_tz::Asia::Dhaka;
use sqlx::PgPool;
use serde::Deserialize;

fn now_dhaka() -> DateTime<FixedOffset> {
    Dhaka.from_utc_datetime(&chrono::Utc::now().naive_utc()).fixed_offset()
}

fn start_of_month_dhaka() -> DateTime<FixedOffset> {
    let now = Dhaka.from_utc_datetime(&chrono::Utc::now().naive_utc());
    Dhaka.with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0).unwrap().fixed_offset()
}

#[derive(Deserialize)]
pub struct TopParams {
    from: Option<DateTime<FixedOffset>>,
    to: Option<DateTime<FixedOffset>>,
    lim: Option<i32>,
}

#[derive(Deserialize)]
pub struct StatsParams {
    from: Option<DateTime<FixedOffset>>,
    to: Option<DateTime<FixedOffset>>,
}

#[derive(Deserialize)]
pub struct HistoryParams {
    from: DateTime<FixedOffset>,
    to: DateTime<FixedOffset>,
}

fn resolve_top_params(params: TopParams) -> (DateTime<FixedOffset>, DateTime<FixedOffset>, i32) {
    let from = params.from.unwrap_or_else(start_of_month_dhaka);
    let to = params.to.unwrap_or_else(now_dhaka);
    let lim = params.lim.unwrap_or(10);
    (from, to, lim)
}

fn resolve_stats_params(params: StatsParams) -> (DateTime<FixedOffset>, DateTime<FixedOffset>) {
    let from = params.from.unwrap_or_else(start_of_month_dhaka);
    let to = params.to.unwrap_or_else(now_dhaka);
    (from, to)
}
pub async fn get_top_artists(State(pool): State<PgPool>, Query(params): Query<TopParams>)
    -> Result<Json<Vec<ListArtist>>, StatusCode> {
        let (from, to, lim) = resolve_top_params(params);
        match query_top_artists(&pool, from, to, lim).await {
            Ok(Some(vector)) => Ok(Json(vector)),
            Ok(None) => Ok(Json(vec![])),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
}

pub async fn get_top_albums(State(pool): State<PgPool>, Query(params): Query<TopParams>)
    -> Result<Json<Vec<ListAlbum>>, StatusCode> {
        let (from, to, lim) = resolve_top_params(params);
        match query_top_albums(&pool, from, to, lim).await {
            Ok(Some(vector)) => Ok(Json(vector)),
            Ok(None) => Ok(Json(vec![])),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
}

pub async fn get_top_tracks(State(pool): State<PgPool>, Query(params): Query<TopParams>)
    -> Result<Json<Vec<ListTrack>>, StatusCode> {
        let (from, to, lim) = resolve_top_params(params);
        match query_top_tracks(&pool, from, to, lim).await {
            Ok(Some(vector)) => Ok(Json(vector)),
            Ok(None) => Ok(Json(vec![])),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
}

pub async fn get_stats(State(pool): State<PgPool>, Query(params): Query<StatsParams>)
    -> Result<Json<Vec<ListStats>>, StatusCode> {
        let (from, to) = resolve_stats_params(params);
        match query_stats(&pool, from, to).await {
            Ok(Some(vector)) => Ok(Json(vector)),
            Ok(None) => Ok(Json(vec![])),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
}

pub async fn get_play_history(State(pool): State<PgPool>, Query(params): Query<HistoryParams>)
    -> Result<Json<Vec<ListPlay>>, StatusCode> {
        match query_play_history(&pool, params.from, params.to).await {
            Ok(Some(vector)) => Ok(Json(vector)),
            Ok(None) => Ok(Json(vec![])),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
}

pub async fn get_recents(State(pool): State<PgPool>)
    -> Result<Json<Vec<ListPlay>>, StatusCode> {
    match query_recents(&pool).await {
        Ok(Some(vector)) => Ok(Json(vector)),
        Ok(None) => Ok(Json(vec![])),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
