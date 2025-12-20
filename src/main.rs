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
    let market:rspotify::model::Market = rspotify::model::Market::FromToken;
    //let mut playlist_id: PlaylistId = PlaylistId::from_id("playlist_id").unwrap();
    let playlist_id: PlaylistId;
    let existing_playlist = playlist_exists
        .items
        .iter()
        .find(|p| p.name == playlist_name);

    let mut tracks_to_add: Vec<PlayableId>=Vec::new();

    if let Some(playlist) = existing_playlist {
        println!("Playlist found: {} - {}", playlist.name, playlist.id);
        playlist_id = playlist.id.clone();

        let current_playlist = spotify.playlist_items_manual(playlist.id.clone(), None, Some(market), None, None).await?;
        println!("Current playlist has {} items", current_playlist.total);
        
        onelibrary.tracks.iter().for_each(|t| {
            let track_id = TrackId::from_id(&t.spotify_id).unwrap();
            let exists = current_playlist.items.iter().any(|item| {
                if let Some(track) = &item.track {
                    if let rspotify::model::PlayableItem::Track(existing_track) = track {
                        return existing_track.id == Some(track_id.clone());
                    }
                }
                false
            });
            if !exists {
                tracks_to_add.push(PlayableId::from(track_id));
            }
        });
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
        playlist_id = playlist.id.clone();
        println!("Created playlist: {} - {}", playlist.name, playlist.id);

        onelibrary
        .tracks
        .iter()
        .for_each(|t| tracks_to_add.push(PlayableId::from(TrackId::from_id(&t.spotify_id).unwrap())));
    }

    if !tracks_to_add.is_empty() {
        spotify.playlist_add_items(playlist_id, tracks_to_add.clone(), None).await?;
        println!("Added {} new tracks to the playlist.", tracks_to_add.len());
    } else {
        println!("No new tracks to add.");
    }

    Ok(())
}
