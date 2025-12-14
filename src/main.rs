use std::fs;
mod track;
// mod spotify_auth;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    //load .env
    dotenv::dotenv().ok();

    let xml_content = fs::read_to_string("example.xml")?;
    let doc = roxmltree::Document::parse(&xml_content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    let mut tracks: Vec<track::Track> = Vec::new();
    // Example: Find nodes with a specific attribute
    
    for node in doc.descendants() {
        if let Some(location) = node.attribute("Location"){
            //&&("POSITION_MARK")
            if location.contains("spotify") {
                let has_position_marker = node.children().any(|child| {
                    child.tag_name().name()=="POSITION_MARK" });

                //continue if no position marker
                if !has_position_marker {
                    continue;
                }

                let spotify_id = location.split("spotify:track:").collect::<Vec<&str>>()[1];

                //date-handling is a bitch
                let date_added = match chrono::NaiveDate::parse_from_str(node.attribute("DateAdded").unwrap_or("1970-01-01"), "%Y-%m-%d") {
                    Ok(date) => date,
                    Err(_) => chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                };

                let track =track::Track {
                    spotify_id: spotify_id.to_string(),
                    date_added: date_added,
                };

                println!("Track struct: spotify_id={}, date_added={}", track.spotify_id, track.date_added);
                tracks.push(track);
            }
        }
    }

    println!("Total found: {}", tracks.len());
    // // Example: Select elements by attribute value
    // println!("\nFind elements where type='track':");
    // for node in doc.descendants() {
    //     if node.attribute("type") == Some("track") {
    //         println!("  Found: {}", node.tag_name().name());
    //     }
    // }

    Ok(())
}