use chrono::NaiveDate;

pub struct Track {
    pub spotify_id: String,
}

pub struct Tracks {
    pub tracks: Vec<Track>,
}

impl Tracks {
    pub fn new(tracks: Vec<Track>) -> Self {
        Self { tracks }
    }

    pub fn fill_from_file(&mut self, filepath: &str, from_date: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let xml_content = std::fs::read_to_string(filepath)?;
        self.fill_from_xml(&xml_content, from_date)
    }

    pub fn fill_from_xml(&mut self, xml_content: &str, from_date: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let doc = roxmltree::Document::parse(xml_content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        let filter_date:NaiveDate = chrono::NaiveDate::parse_from_str(&from_date.unwrap_or("1970-01-01".to_string()), "%Y-%m-%d").unwrap();
        
        println!("Filtering tracks from date: {}", filter_date);
        
        for node in doc.descendants() {
            if let Some(location) = node.attribute("Location") {
                if location.contains("spotify") {
                    let has_position_marker = node.children().any(|child| {
                        child.tag_name().name() == "POSITION_MARK"
                    });

                    if !has_position_marker {
                        continue;
                    }

                    let date_added = chrono::NaiveDate::parse_from_str(node.attribute("DateAdded").unwrap_or("1970-01-01"), "%Y-%m-%d").unwrap();

                    if date_added < filter_date {
                        continue;
                    }

                    let spotify_id = location
                        .split("spotify:track:")
                        .nth(1)
                        .unwrap_or("")
                        .to_string();

                    
                    let new_track = Track {
                        spotify_id,
  
                    };

                    self.tracks.push(new_track);
                }
            }
        }
        Ok(())
    }  }