use chrono::{DateTime, FixedOffset};
use sqlx::{
    PgPool,
    query_as
};
use serde::Serialize;

#[derive(sqlx::FromRow, Serialize)]
pub struct ListArtist {
    name: String,
    play_count: i64,
    play_time: i64, // minutes
    first_listened: DateTime<FixedOffset>,
    first_listened_track: String
}

#[derive(sqlx::FromRow, Serialize)]
pub struct ListAlbum {
    name: String, 
    play_count: i64, 
    play_time: i64, 
    artists: String, // concatenated
    first_listened: DateTime<FixedOffset>,
    first_listened_track: String
}

#[derive(sqlx::FromRow, Serialize)]
pub struct ListTrack {
    name: String,
    play_count: i64,
    play_time: i64, 
    artists: String,
    album: String,
    first_listened: DateTime<FixedOffset>
}

#[derive(sqlx::FromRow, Serialize)]
pub struct ListPlay {
    name: String,
    artists: String, 
    album: String,
    played_at: DateTime<FixedOffset>
}

#[derive(sqlx::FromRow, Serialize)]
pub struct ListStats {
    total_plays: i64,
    unique_tracks: i64, 
    unique_artists: i64,
    unique_albums: i64,
    total_playtime: String,
    avg_release_year: String
}

pub async fn query_top_artists(pool: &PgPool, from: DateTime<FixedOffset>, to: DateTime<FixedOffset>, lim: i32)
    -> Result<Option<Vec<ListArtist>>, sqlx::Error> {
    let artists: Vec<ListArtist> = query_as::<_, ListArtist>(
        "WITH artist_stats AS (
            SELECT 
                art.id AS artist_id,
                art.name,
                COUNT(p.id) AS play_count,
                SUM(t.duration) / 1000 / 60 AS play_time
            FROM plays p
            JOIN tracks t ON p.track_id = t.id
            JOIN track_artists ta ON t.id = ta.track_id
            JOIN artists art ON ta.artist_id = art.id
            WHERE p.played_at >= $1 AND p.played_at < $2
            GROUP BY art.id, art.name
            ORDER BY play_count DESC
            LIMIT $3
        ),
        all_time_firsts AS (
            SELECT DISTINCT ON (art.id)
                art.id AS artist_id,
                p.played_at AS first_listened,
                t.name AS first_listened_track
            FROM plays p
            JOIN tracks t ON p.track_id = t.id
            JOIN track_artists ta ON t.id = ta.track_id
            JOIN artists art ON ta.artist_id = art.id
            WHERE art.id IN (SELECT artist_id FROM artist_stats)
            ORDER BY art.id, p.played_at ASC
        )
        SELECT 
            s.name,
            s.play_count,
            s.play_time,
            f.first_listened,
            f.first_listened_track
        FROM artist_stats s
        JOIN all_time_firsts f ON s.artist_id = f.artist_id
        ORDER BY s.play_count DESC")
        .bind(from)
        .bind(to)
        .bind(lim)
        .fetch_all(pool)
        .await?;
    if artists.is_empty() {
        return Ok(None)
    }
    Ok(Some(artists))
}

pub async fn query_top_albums(pool: &PgPool, from: DateTime<FixedOffset>, to: DateTime<FixedOffset>, lim: i32)
    -> Result<Option<Vec<ListAlbum>>, sqlx::Error> {
    let albums: Vec<ListAlbum> = query_as::<_, ListAlbum>(
        "WITH album_stats AS (
            SELECT 
                a.id AS album_id,
                a.name,
                COUNT(p.id) AS play_count,
                SUM(t.duration) / 1000 / 60 AS play_time,
                STRING_AGG(DISTINCT art.name, ', ' ORDER BY art.name) AS artists
            FROM plays p
            JOIN tracks t ON p.track_id = t.id
            JOIN track_albums ta ON t.id = ta.track_id
            JOIN albums a ON ta.album_id = a.id
            JOIN album_artists aa ON a.id = aa.album_id
            JOIN artists art ON aa.artist_id = art.id
            WHERE p.played_at >= $1 AND p.played_at < $2
            GROUP BY a.id, a.name
            ORDER BY play_count DESC
            LIMIT $3
        ),
        album_firsts AS (
            SELECT DISTINCT ON (ta.album_id)
                ta.album_id,
                p.played_at AS first_listened,
                t.name AS first_listened_track
            FROM plays p
            JOIN tracks t ON p.track_id = t.id
            JOIN track_albums ta ON t.id = ta.track_id
            WHERE ta.album_id IN (SELECT album_id FROM album_stats)
            ORDER BY ta.album_id, p.played_at ASC
        )
        SELECT 
            s.name,
            s.play_count,
            s.play_time,
            s.artists,
            f.first_listened,
            f.first_listened_track
        FROM album_stats s
        JOIN album_firsts f ON s.album_id = f.album_id
        ORDER BY s.play_count DESC")
        .bind(from)
        .bind(to)
        .bind(lim)
        .fetch_all(pool)
        .await?;
    if albums.is_empty() {
        return Ok(None)
    }
    Ok(Some(albums))
}

pub async fn query_top_tracks(pool: &PgPool, from: DateTime<FixedOffset>, to: DateTime<FixedOffset>, lim: i32)
    -> Result<Option<Vec<ListTrack>>, sqlx::Error> {
    let tracks: Vec<ListTrack> = query_as::<_, ListTrack>(
        "WITH top_track_ids AS (
            SELECT 
                track_id,
                COUNT(*) AS play_count,
                SUM(t.duration) / 1000 / 60 AS play_time
            FROM plays p
            JOIN tracks t ON p.track_id = t.id
            WHERE p.played_at >= $1 AND p.played_at < $2
            GROUP BY track_id
            ORDER BY play_count DESC
            LIMIT $3
        ),
        track_metadata AS (
            SELECT 
                t.id AS track_id,
                t.name,
                STRING_AGG(DISTINCT art.name, ', ' ORDER BY art.name) AS artists,
                MIN(a.name) AS album
            FROM tracks t
            JOIN track_artists ta ON t.id = ta.track_id
            JOIN artists art ON ta.artist_id = art.id
            LEFT JOIN track_albums tal ON t.id = tal.track_id
            LEFT JOIN albums a ON tal.album_id = a.id
            WHERE t.id IN (SELECT track_id FROM top_track_ids)
            GROUP BY t.id, t.name
        ),
        track_firsts AS (
            SELECT DISTINCT ON (track_id)
                track_id,
                played_at AS first_listened
            FROM plays
            WHERE track_id IN (SELECT track_id FROM top_track_ids)
            ORDER BY track_id, played_at ASC
        )
        SELECT 
            m.name,
            s.play_count,
            s.play_time,
            m.artists,
            m.album,
            f.first_listened
        FROM top_track_ids s
        JOIN track_metadata m ON s.track_id = m.track_id
        JOIN track_firsts f ON s.track_id = f.track_id
        ORDER BY s.play_count DESC")
        .bind(from)
        .bind(to)
        .bind(lim)
        .fetch_all(pool)
        .await?;
    if tracks.is_empty() {
        return Ok(None)
    }
    Ok(Some(tracks))
}

pub async fn query_play_history(pool: &PgPool, from: DateTime<FixedOffset>, to: DateTime<FixedOffset>)
    -> Result<Option<Vec<ListPlay>>, sqlx::Error> {
    let plays: Vec<ListPlay> = query_as::<_, ListPlay>(
        "SELECT 
            t.name,
            STRING_AGG(DISTINCT art.name, ', ' ORDER BY art.name) AS artists,
            MIN(a.name) AS album,
            p.played_at
        FROM plays p
        JOIN tracks t ON p.track_id = t.id
        JOIN track_artists ta ON t.id = ta.track_id
        JOIN artists art ON ta.artist_id = art.id
        LEFT JOIN track_albums tal ON t.id = tal.track_id
        LEFT JOIN albums a ON tal.album_id = a.id
        WHERE p.played_at >= $1 AND p.played_at < $2
        GROUP BY t.name, p.played_at
        ORDER BY p.played_at DESC")
        .bind(from)
        .bind(to)
        .fetch_all(pool)
        .await?;
    if plays.is_empty() {
        return Ok(None)
    }
    Ok(Some(plays))
}

pub async fn query_recents(pool: &PgPool)
    -> Result<Option<Vec<ListPlay>>, sqlx::Error> {
    let plays: Vec<ListPlay> = query_as::<_, ListPlay>(
        "SELECT
            t.name,
            STRING_AGG(DISTINCT art.name, ', ' ORDER BY art.name) AS artists,
            MIN(a.name) AS album,
            p.played_at
        FROM plays p
        JOIN tracks t ON p.track_id = t.id
        JOIN track_artists ta ON t.id = ta.track_id
        JOIN artists art ON ta.artist_id = art.id
        LEFT JOIN track_albums tal ON t.id = tal.track_id
        LEFT JOIN albums a ON tal.album_id = a.id
        GROUP BY t.name, p.played_at
        ORDER BY p.played_at DESC
        LIMIT 25")
        .fetch_all(pool)
        .await?;
    if plays.is_empty() {
        return Ok(None)
    }
    Ok(Some(plays))
}

pub async fn query_stats(pool: &PgPool, from: DateTime<FixedOffset>, to: DateTime<FixedOffset>)
    -> Result<Option<Vec<ListStats>>, sqlx::Error> {
    let stats: Vec<ListStats> = query_as::<_, ListStats>(
        "WITH filtered_plays AS (
            SELECT 
                p.track_id,
                t.duration,
                talb.album_id,
                NULLIF(SUBSTRING(a.release_date FROM 1 FOR 4), '')::INT AS release_year
            FROM plays p
            JOIN tracks t ON p.track_id = t.id
            LEFT JOIN track_albums talb ON p.track_id = talb.track_id
            LEFT JOIN albums a ON talb.album_id = a.id
            WHERE p.played_at::DATE BETWEEN $1::DATE AND $2::DATE
        ),
        artist_counts AS (
            SELECT COUNT(DISTINCT ta.artist_id) as unique_artists
            FROM plays p
            JOIN track_artists ta ON p.track_id = ta.track_id
            WHERE p.played_at::DATE BETWEEN $1::DATE AND $2::DATE
        )
        SELECT 
            COUNT(*) AS total_plays,
            COUNT(DISTINCT track_id) AS unique_tracks,
            (SELECT unique_artists FROM artist_counts) AS unique_artists,
            COUNT(DISTINCT album_id) AS unique_albums,
            
            COALESCE(FLOOR(SUM(duration) / 3600000), 0) || 'h ' || 
            COALESCE(FLOOR((SUM(duration) % 3600000) / 60000), 0) || 'm' AS total_playtime,

            COALESCE(ROUND(AVG(release_year))::TEXT, '0') AS avg_release_year

        FROM filtered_plays;")
        .bind(from)
        .bind(to)
        .fetch_all(pool)
        .await?;
    if stats.is_empty() {
        return Ok(None)
    }
    Ok(Some(stats))
}
