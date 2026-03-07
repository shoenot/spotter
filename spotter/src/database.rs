use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug)]
pub struct Artist {
    pub id:   String,
    pub name: String,
    pub image_link: String,
    pub link: String,
}

#[derive(Debug)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub album_type: String,
    pub release_date: String,
    pub image_link: String,
    pub link: String,
}

#[derive(Debug)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub duration: i32,
    pub popularity: i32,
    pub link: String,
}

#[derive(Debug)]
pub struct Play {
    pub track_id: String,
    pub played_at: DateTime<Utc>
}

impl Artist {
    pub async fn insert(&self, pool: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO artists (id, name, image_url, artist_url)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (id) DO NOTHING",
            self.id,
            self.name,
            self.image_link,
            self.link,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl Album {
    pub async fn insert(&self, pool: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO albums (id, name, album_type, release_date, image_url, album_url)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO NOTHING",
            self.id,
            self.name,
            self.album_type,
            self.release_date,
            self.image_link,
            self.link,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl Track {
    pub async fn insert(&self, pool: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO tracks (id, name, duration, popularity, track_url)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (id) DO NOTHING",
            self.id,
            self.name,
            self.duration,
            self.popularity as i32,  // sqlx expects i32 for INTEGER, your struct has u32
            self.link,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl Play {
    pub async fn insert(&self, pool: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO plays (track_id, played_at)
             VALUES ($1, $2)
             ON CONFLICT (played_at) DO NOTHING",
            self.track_id,
            self.played_at,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

// junction tables take ids directly rather than whole structs
pub async fn insert_album_artist(
    pool: &PgPool,
    album_id: &str,
    artist_id: &str,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO album_artists (album_id, artist_id)
         VALUES ($1, $2)
         ON CONFLICT DO NOTHING",
        album_id,
        artist_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_track_artist(
    pool: &PgPool,
    track_id: &str,
    artist_id: &str,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO track_artists (track_id, artist_id)
         VALUES ($1, $2)
         ON CONFLICT DO NOTHING",
        track_id,
        artist_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_track_album(
    pool: &PgPool,
    track_id: &str,
    album_id: &str,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO track_albums (track_id, album_id)
         VALUES ($1, $2)
         ON CONFLICT DO NOTHING",
        track_id,
        album_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_last_played_at(pool: & PgPool) -> Option<DateTime<Utc>> {
    sqlx::query_scalar!("SELECT MAX(played_at) FROM plays")
        .fetch_one(pool)
        .await
        .ok()
        .flatten()
}

