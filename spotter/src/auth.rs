use rspotify::{
    prelude::*,
    scopes, AuthCodeSpotify, Config, Credentials, OAuth,
    ClientError
};

pub async fn auth_spotify() -> Result<AuthCodeSpotify, ClientError> {
    let creds = Credentials::from_env().expect("RSPOTIFY_CLIENT_ID and RSPOTIFY_CLIENT_SECRET must be set");
    let oauth = OAuth {
        redirect_uri: "http://127.0.0.1:8888/callback".to_string(),
        scopes: scopes!("user-read-recently-played"),
        ..Default::default()
    };
    let config = Config {
        token_refreshing: true,
        token_cached: true,
        ..Default::default()
    };

    let spotify = AuthCodeSpotify::with_config(creds, oauth, config);

    let needs_auth = match spotify.read_token_cache(true).await {
        Ok(_) => {
            // cache loaded, check it actually has a token
            spotify.get_token().lock().await.unwrap().is_none()
        }
        Err(_) => true,  // no cache or failed to read
    };

    if needs_auth {
        let url = spotify
            .get_authorize_url(false)
            .expect("Failed to build authorize URL");

        println!("Open this URL in your browser to authenticate:");
        println!("\n{}\n", url);
        println!("After authorizing, paste the full redirect URL here:");

        let mut redirect_url = String::new();
        std::io::stdin()
            .read_line(&mut redirect_url)
            .expect("Failed to read redirect URL");
        let redirect_url = redirect_url.trim();

        let code = spotify 
            .parse_response_code(redirect_url)
            .expect("Could not parse code from redirect URL — make sure you pasted the full URL");
        
        spotify.request_token(&code).await?;
    }

    Ok(spotify)
}
