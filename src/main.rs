
mod onelibrary;
mod spotify_auth;

// mod spotify_auth;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    //load .env
    dotenv::dotenv().ok();

    spotify_auth::authenticate_spotify().await;
    
    let filepath="example.xml";

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

    println!("Total found: {}", onelibrary.tracks.len());

    Ok(())
}