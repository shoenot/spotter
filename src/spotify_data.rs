use rspotify::{
    prelude::*,
    AuthCodeSpotify,
    model::{
        CursorBasedPage,
        PlayHistory,
        SimplifiedArtist,
        ArtistId,
        TimeLimits
    },
    clients::BaseClient,
    ClientError
};
use crate::database;

async fn get_spotify_recents(spotify: &AuthCodeSpotify, number: u32, cursor: Option<TimeLimits>) 
    -> Result<CursorBasedPage<PlayHistory>, ClientError> { 
    spotify
        .current_user_recently_played(Some(number), cursor)
        .await
}

async fn get_artist_image_link(spotify: &AuthCodeSpotify, id: &ArtistId<'_>) 
    -> Result<Option<String>, ClientError> {  
    let artist = spotify.artist(id.clone()).await?;
    Ok(artist.images.first().map(|img| img.url.clone()))
}

pub struct PlayData {
    pub artists: Vec<database::Artist>,
    pub album: database::Album,
    pub track: database::Track,
    pub play: database::Play,
}

async fn build_artist(spotify: &AuthCodeSpotify,artist: &SimplifiedArtist) 
    -> Option<database::Artist> {
    let id = artist.id.as_ref()?;  

    let image_link = get_artist_image_link(spotify, id)
        .await
        .ok()           // convert Result to Option, None if the API call fails
        .flatten()      // flatten Option<Option<String>> to Option<String>
        .unwrap_or_default();  // fall back to empty string if no image

    Some(database::Artist {
        id: id.uri(),         
        name: artist.name.clone(),  
        image_link,
        link: artist.href.clone().unwrap_or_default(), 
    })
}

pub async fn get_recent_plays(spotify: &AuthCodeSpotify, number: u32, cursor: Option<TimeLimits>) -> Result<Vec<PlayData>, ClientError> {
    let recents = get_spotify_recents(spotify, number, cursor).await?;
    let mut recent_plays_data = Vec::new();

    for item in recents.items { 
        let track_id = match item.track.id {
            Some(ref id) => id.uri(),
            None => continue, 
        };

        let track = database::Track {
            id: track_id.clone(),
            name: item.track.name.clone(),
            duration: item.track.duration.num_milliseconds() as i32, 
            popularity: item.track.popularity as i32,
            link: item.track.href.clone().unwrap_or_default(),
        };

        // need .await and futures::future::join_all to await a vec of futures
        let artist_futures: Vec<_> = item.track.artists
            .iter()
            .map(|artist| build_artist(spotify, artist))
            .collect();
        let artists: Vec<database::Artist> = futures::future::join_all(artist_futures)
            .await
            .into_iter()
            .flatten()  // removes None values (artists with no id)
            .collect();

        let album_id = item.track.album.id
            .as_ref()
            .map(|id| id.uri())
            .unwrap_or_default();

        let album = database::Album {
            id: album_id,
            name: item.track.album.name.clone(),
            album_type: item.track.album.album_type
                .map(|t| format!("{:?}", t))
                .unwrap_or_default(),
            release_date: item.track.album.release_date.clone().unwrap_or_default(),
            image_link: item.track.album.images
                .first()
                .map(|img| img.url.clone())
                .unwrap_or_default(),
            link: item.track.album.href.clone().unwrap_or_default(),
        };

        let play = database::Play {
            id: 0, 
            track_id: track.id.clone(),
            played_at: item.played_at,
        };

        recent_plays_data.push(PlayData { artists, album, track, play });
    }

    Ok(recent_plays_data)
}
