use tokio::sync::oneshot;
use std::sync::{Arc, Mutex};
use axum::{
    extract::{Query, State},
    response::Html,
    routing::get,
    Router,
};
use std::collections::HashMap;
use rspotify::{
    prelude::*,
    scopes, AuthCodeSpotify, Config, Credentials, OAuth,
    ClientError
};

type CodeSender = Arc<Mutex<Option<oneshot::Sender<String>>>>;

async fn callback_handler(
    State(tx): State<CodeSender>,
    Query(params): Query<HashMap<String, String>>,
) -> Html<&'static str> {
    if let Some(code) = params.get("code") {
        if let Some(sender) = tx.lock().unwrap().take() {
            let _ = sender.send(code.clone());
        }
        Html("<h2>authenticated</h2><p>token get! you can close this tab :D</p>")
    } else {
        Html("<h2>error</h2><p>no code parameter in callback url :(</p>")
    }
}

async fn wait_for_callback() -> Result<String, Box<dyn std::error::Error>> {
    let (tx, rx) = oneshot::channel::<String>();
    let state: CodeSender = Arc::new(Mutex::new(Some(tx)));

    let app = Router::new()
        .route("/callback", get(callback_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await?;
    println!("Listening on 0.0.0.0:8888 for OAuth callback...");

    tokio::spawn(async {
        axum::serve(listener, app).await.ok();
    });

    let code = rx.await?;
    println!("Token get!");
    Ok(code)
}

pub async fn auth_spotify() -> Result<AuthCodeSpotify, ClientError> {
    let creds = Credentials::from_env()
        .expect("RSPOTIFY_CLIENT_ID and RSPOTIFY_CLIENT_SECRET must be set");

    let redirect_uri = std::env::var("RSPOTIFY_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:8888/callback".to_string());

    let cache_path = std::path::PathBuf::from(
        std::env::var("RSPOTIFY_CACHE_PATH")
            .unwrap_or_else(|_| ".spotify_token_cache.json".to_string())
    );

    let oauth = OAuth {
        redirect_uri,
        scopes: scopes!("user-read-recently-played"),
        ..Default::default()
    };
    let config = Config {
        token_refreshing: true,
        token_cached: true,
        cache_path: cache_path.clone(),
        ..Default::default()
    };

    let spotify = AuthCodeSpotify::with_config(creds, oauth, config);

    let needs_auth = if cache_path.exists() {
        match spotify.read_token_cache(true).await {
            Ok(Some(token)) => {
                *spotify.get_token().lock().await.unwrap() = Some(token);
                false
            }
            _ => true,
        }
    } else {
        true
    };

    if needs_auth {
        let url = spotify
            .get_authorize_url(false)
            .expect("Failed to build authorize URL");

        println!("Open this URL in your browser to authenticate:");
        println!("\n{}\n", url);

        let code = wait_for_callback().await
            .expect("Failed to receive OAuth callback");

        spotify.request_token(&code).await?;
        spotify.write_token_cache().await
            .expect("Failed to write token cache");
    }

    Ok(spotify)
}
