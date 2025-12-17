use chrono::NaiveDate;

pub struct Track {
    pub spotify_id: String,
    pub date_added: NaiveDate,
}

pub struct Tracks {
    pub tracks: Vec<Track>,
}

impl Tracks {
    pub fn new(tracks: Vec<Track>) -> Self {
        Self { tracks }
    }

    pub fn fill_from_file(&mut self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        let xml_content = std::fs::read_to_string(filepath)?;
        self.fill_from_xml(&xml_content)
    }
    
    pub fn fill_from_xml(&mut self, xml_content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let doc = roxmltree::Document::parse(xml_content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        for node in doc.descendants() {
            if let Some(location) = node.attribute("Location") {
                if location.contains("spotify") {
                    let has_position_marker = node.children().any(|child| {
                        child.tag_name().name() == "POSITION_MARK"
                    });

                    if !has_position_marker {
                        continue;
                    }

                    let spotify_id = location
                        .split("spotify:track:")
                        .nth(1)
                        .unwrap_or("")
                        .to_string();

                    let date_added = match chrono::NaiveDate::parse_from_str(node.attribute("DateAdded").unwrap_or("1970-01-01"), "%Y-%m-%d") {
                        Ok(date) => date,
                        Err(_) => chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                    };

                    let new_track = Track {
                        spotify_id,
                        date_added,
                    };

                    self.tracks.push(new_track);
                }
            }
        }
        Ok(())
    }  }