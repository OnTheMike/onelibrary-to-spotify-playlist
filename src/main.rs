use rspotify::{
    model::{PlayableId, PlayableItem, PlaylistId, TrackId, UserId},
    prelude::{BaseClient, OAuthClient, Id},
};
use clap::Parser;
use std::collections::HashSet;

mod onelibrary;
mod spotify_auth;

const MAX_PLAYLISTS: u32 = 50;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct ToolArgs {
    /// Name of the playlist to add the tracks to
    #[arg(short, long, default_value = "DJ Selection")]
    playlist_name: String,

    /// Path to the OneLibrary XML file
    #[arg(short, long)]
    file: String,

    /// Date to filter tracks from (YYYY-MM-DD)
    #[arg(short = 'd', long)]
    from_date: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let cli = ToolArgs::parse();

    // Validate file exists
    if !std::path::Path::new(&cli.file).exists() {
        return Err(format!("File '{}' does not exist", cli.file).into());
    }

    // Load tracks from OneLibrary
    log::info!("Loading tracks from {}", cli.file);
    let mut onelibrary = onelibrary::Tracks::new(Vec::new());
    onelibrary.fill_from_file(&cli.file, cli.from_date)?;
    log::info!("Successfully loaded {} tracks", onelibrary.tracks.len());

    if onelibrary.tracks.is_empty() {
        log::warn!("No tracks found in file. Exiting.");
        return Ok(());
    }

    // Authenticate with Spotify
    log::info!("Authenticating with Spotify...");
    let spotify = spotify_auth::authenticate_spotify().await?;
    let spotify_user = spotify.current_user().await?;
    log::info!(
        "Authenticated as user: {}",
        spotify_user.display_name.unwrap_or_default()
    );

    // Get or create playlist
    let playlist_id = get_or_create_playlist(&spotify, &spotify_user.id, &cli.playlist_name).await?;

    // Add new tracks to playlist
    add_new_tracks_to_playlist(&spotify, playlist_id, &onelibrary.tracks).await?;

    log::info!("Successfully completed!");
    Ok(())
}

/// Gets an existing playlist or creates a new one
async fn get_or_create_playlist(
    spotify: &rspotify::AuthCodeSpotify,
    user_id: &UserId<'_>,
    playlist_name: &str,
) -> Result<PlaylistId<'static>, Box<dyn std::error::Error>> {
    // Fetch user's playlists
    let playlists = spotify
        .user_playlists_manual(user_id.clone(), Some(MAX_PLAYLISTS), Some(0))
        .await?;

    // Check if playlist already exists
    if let Some(playlist) = playlists.items.iter().find(|p| p.name == playlist_name) {
        log::info!("Found existing playlist: {} ({})", playlist.name, playlist.id);
        return Ok(playlist.id.clone());
    }

    // Create new playlist
    log::info!("Creating new playlist: {}", playlist_name);
    let new_playlist = spotify
        .user_playlist_create(
            user_id.clone(),
            playlist_name,
            Some(false),
            Some(false),
            None,
        )
        .await?;

    log::info!("Created playlist: {} ({})", new_playlist.name, new_playlist.id);
    Ok(new_playlist.id)
}

/// Adds new tracks to a Spotify playlist, avoiding duplicates
async fn add_new_tracks_to_playlist(
    spotify: &rspotify::AuthCodeSpotify,
    playlist_id: PlaylistId<'_>,
    tracks_to_process: &[onelibrary::Track],
) -> Result<(), Box<dyn std::error::Error>> {
    // Fetch ALL playlist items (handle pagination)
    let market = rspotify::model::Market::FromToken;
    const PAGE_SIZE: u32 = 50;
    
    log::info!("Fetching all existing playlist items with pagination...");
    
    let mut existing_ids: HashSet<String> = HashSet::new();
    let mut offset = 0;
    let mut total;
    
    loop {
        let page = spotify
            .playlist_items_manual(playlist_id.clone(), None, Some(market), Some(PAGE_SIZE), Some(offset))
            .await?;
        
        total = page.total as u32;
        log::debug!("Fetched page with {} items (offset: {}, total: {})", page.items.len(), offset, total);
        
        // Extract track IDs from this page
        for item in page.items.iter() {
            if let Some(PlayableItem::Track(track)) = &item.track {
                if let Some(id) = track.id.as_ref() {
                    existing_ids.insert(id.id().to_string());
                }
            }
        }
        
        // Check if we've fetched all items
        offset += PAGE_SIZE;
        if page.items.is_empty() || offset >= total {
            log::debug!("Finished fetching all {} playlist items", existing_ids.len());
            break;
        }
    }

    log::info!("Playlist has {} total items", total);

    // Filter new tracks
    let mut new_tracks = Vec::new();
    let mut skipped = 0;

    for track in tracks_to_process {
        // Try to parse the track ID
        match TrackId::from_id(&track.spotify_id) {
            Ok(track_id) => {
                if existing_ids.contains(track_id.id()) {
                    skipped += 1;
                } else {
                    new_tracks.push(PlayableId::from(track_id));
                }
            }
            Err(_) => {
                log::warn!("Invalid Spotify ID: {}", track.spotify_id);
                skipped += 1;
            }
        }
    }

    log::info!(
        "Found {} new tracks to add (skipped {} duplicates/invalid)",
        new_tracks.len(),
        skipped
    );

    // Add tracks to playlist in batches (Spotify API limit is 100 per request)
    if !new_tracks.is_empty() {
        const BATCH_SIZE: usize = 100;
        for chunk in new_tracks.chunks(BATCH_SIZE) {
            spotify
                .playlist_add_items(playlist_id.clone(), chunk.to_vec(), None)
                .await?;
            log::debug!("Added {} tracks to playlist", chunk.len());
        }
        log::info!("Successfully added {} new tracks!", new_tracks.len());
    } else {
        log::info!("No new tracks to add.");
    }

    Ok(())
}
