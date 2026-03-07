mod auth;
mod spotify_data;
mod database;

use dotenvy;
use sqlx::PgPool;
use tokio::time::{interval, Duration};
use rspotify::model::TimeLimits;
use crate::auth::auth_spotify;
use crate::spotify_data::{get_recent_plays, PlayData};
use crate::database::*;

async fn run_inserts(pool: &PgPool, recent_songs: Vec<PlayData>) 
    -> Result<(), Box<dyn std::error::Error>> {
    for song in recent_songs {
        let results: Vec<_> = futures::future::join_all(
            song.artists.iter().map(|a| a.insert(pool))
        ).await;

        if results.iter().any(|r| r.is_err())
            || song.album.insert(pool).await.is_err()
            || song.track.insert(pool).await.is_err()
            || song.play.insert(pool).await.is_err()
        {
            eprintln!("Skipping {} at {} - insert failed", song.track.name, song.play.played_at);
            continue;
        }

        for artist in &song.artists {
            insert_album_artist(pool, &song.album.id, &artist.id).await?;
            insert_track_artist(pool, &song.track.id, &artist.id).await?;
        }
        insert_track_album(pool, &song.track.id, &song.album.id).await?;
        println!("Inserted {} :)", song.track.name);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let spotify= auth_spotify().await?;

    let mut ticker = interval(Duration::from_secs(300));
    
    loop {
        ticker.tick().await;

        let after = get_last_played_at(&pool).await;
        let cursor = after.map(|t| TimeLimits::After(t));

        let recent_songs = match get_recent_plays(&spotify, 50, cursor).await {
            Ok(songs) => songs,
            Err(e) => {
                eprintln!("Poll failed: {e}");
                continue;
            }
        };

        if let Err(e) = run_inserts(&pool, recent_songs).await {
            eprintln!("An error occurred: {e}");
            continue;
        };
    }
}
