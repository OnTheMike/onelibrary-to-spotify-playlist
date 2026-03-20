use rspotify::{prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth};

/// Authenticates with Spotify using OAuth flow
///
/// # Errors
/// Returns an error if credentials are not found in environment or OAuth fails
pub async fn authenticate_spotify() -> Result<AuthCodeSpotify, Box<dyn std::error::Error>> {
    let creds = Credentials::from_env()
        .ok_or("Failed to get Spotify credentials from environment")?;

    let scopes = scopes!(
        "playlist-modify-public",
        "playlist-modify-private",
        "playlist-read-private",
        "user-library-read"
    );

    let oauth = OAuth::from_env(scopes)
        .ok_or("Failed to get Spotify OAuth from environment")?;

    let spotify = AuthCodeSpotify::new(creds, oauth);

    // Obtaining the access token
    let url = spotify.get_authorize_url(false)
        .map_err(|e| format!("Failed to get authorization URL: {}", e))?;
    
    spotify.prompt_for_token(&url).await
        .map_err(|e| format!("Failed to authenticate: {}", e))?;

    Ok(spotify)
}