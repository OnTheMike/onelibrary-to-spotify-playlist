use rspotify::{
    //model::{AdditionalType, Country, Market},
    prelude::*,
    scopes, AuthCodeSpotify, Credentials, OAuth,
};

pub async fn authenticate_spotify() -> AuthCodeSpotify {
    let creds = Credentials::from_env().expect("Failed to get Spotify credentials from environment");
    
    let scopes = scopes!(
        "playlist-modify-public",
        "playlist-modify-private",
        "playlist-read-private",
        "user-library-read"
    );

    let oauth = OAuth::from_env(scopes).expect("Failed to get Spotify OAuth from environment");

    let spotify = AuthCodeSpotify::new(creds, oauth);

    // Obtaining the access token
    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url).await.unwrap();

    spotify
}