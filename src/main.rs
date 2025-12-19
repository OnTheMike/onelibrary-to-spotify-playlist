use std::process::id;

use rspotify::{
    model::{PlayableId, PlaylistId, TrackId},
    prelude::{BaseClient, OAuthClient},
};

mod onelibrary;
mod spotify_auth;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spotify = spotify_auth::authenticate_spotify().await;
    let playlist_name = "DJ Selection";
    let filepath = "example.xml";

    if !std::path::Path::new(filepath).exists() {
        println!("File {} does not exist.", filepath);
        return Ok(());
    }

    let mut onelibrary = onelibrary::Tracks::new(Vec::new());

    //let xml_content: &str = &fs::read_to_string(filepath)?;
    match onelibrary.fill_from_file(filepath) {
        Ok(_) => println!("Tracks filled successfully."),
        Err(e) => println!("Error filling tracks: {}", e),
    }

    let spotify_user = spotify.current_user().await?;

    let playlist_exists = spotify
        .user_playlists_manual(spotify_user.id.clone(), Some(50), Some(0))
        .await
        .unwrap();

    //println!("Found {} playlists", playlist_exists.total);

    //let mut playlist_id: PlaylistId = PlaylistId::from_id("playlist_id").unwrap();
    let playlist_id: PlaylistId;
    let existing_playlist = playlist_exists
        .items
        .iter()
        .find(|p| p.name == playlist_name);

    if let Some(playlist) = existing_playlist {
        println!("Playlist found: {} - {}", playlist.name, playlist.id);
        playlist_id = playlist.id.clone();
        //playlist.tracks;
    } else {
        let playlist = spotify
            .user_playlist_create(
                spotify_user.id.clone(),
                playlist_name,
                Some(false),
                Some(false),
                None,
            )
            .await?;

        println!("Created playlist: {} - {}", playlist.name, playlist.id);

        playlist_id = playlist.id.clone();
    }

    //only retain tracks that are not already in the playlist

    
    let items: Vec<PlayableId> = onelibrary
        .tracks
        .iter()
        .map(|t| PlayableId::from(TrackId::from_id(&t.spotify_id).unwrap()))
        .collect();

    spotify.playlist_add_items(playlist_id, items.clone(), None).await?;

    println!("Total found: {}", onelibrary.tracks.len());

    Ok(())
}
