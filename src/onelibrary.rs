use chrono::NaiveDate;
use std::fmt;

/// Represents a track from OneLibrary
#[derive(Debug, Clone)]
pub struct Track {
    pub spotify_id: String,
}

/// A collection of tracks parsed from OneLibrary XML
pub struct Tracks {
    pub tracks: Vec<Track>,
}

impl Tracks {
    /// Creates a new empty Tracks collection
    pub fn new(tracks: Vec<Track>) -> Self {
        Self { tracks }
    }

    /// Fills tracks from a OneLibrary XML file
    ///
    /// # Arguments
    /// * `filepath` - Path to the XML file
    /// * `from_date` - Optional date filter (YYYY-MM-DD format)
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed
    pub fn fill_from_file(
        &mut self,
        filepath: &str,
        from_date: Option<String>,
    ) -> Result<(), TrackParseError> {
        let xml_content = std::fs::read_to_string(filepath)
            .map_err(|e| TrackParseError::FileError(filepath.to_string(), e.to_string()))?;
        self.fill_from_xml(&xml_content, from_date)
    }

    /// Parses tracks from XML content
    ///
    /// # Arguments
    /// * `xml_content` - The XML content as a string
    /// * `from_date` - Optional date filter (YYYY-MM-DD format)
    ///
    /// # Errors
    /// Returns an error if XML parsing or date parsing fails
    pub fn fill_from_xml(
        &mut self,
        xml_content: &str,
        from_date: Option<String>,
    ) -> Result<(), TrackParseError> {
        let doc = roxmltree::Document::parse(xml_content)
            .map_err(|e| TrackParseError::XmlParseError(e.to_string()))?;

        let filter_date = parse_date(&from_date.unwrap_or_else(|| "1970-01-01".to_string()))?;

        log::info!("Filtering tracks from date: {}", filter_date);

        for node in doc.descendants() {
            if let Some(location) = node.attribute("Location") {
                if !location.contains("spotify") {
                    continue;
                }

                // Check for POSITION_MARK
                let has_position_marker = node.children().any(|child| child.tag_name().name() == "POSITION_MARK");
                if !has_position_marker {
                    continue;
                }

                // Parse date
                let date_added_str = node.attribute("DateAdded").unwrap_or("1970-01-01");
                let date_added = match parse_date(date_added_str) {
                    Ok(date) => date,
                    Err(e) => {
                        log::warn!("Failed to parse date for track {}: {}", location, e);
                        continue;
                    }
                };

                if date_added < filter_date {
                    continue;
                }

                // Extract Spotify ID
                let spotify_id = extract_spotify_id(location);
                if spotify_id.is_empty() {
                    log::warn!("Could not extract Spotify ID from: {}", location);
                    continue;
                }

                let new_track = Track { spotify_id };
                self.tracks.push(new_track);
            }
        }

        log::info!("Parsed {} tracks from XML", self.tracks.len());
        Ok(())
    }
}

/// Parses a date string in YYYY-MM-DD format
fn parse_date(date_str: &str) -> Result<NaiveDate, TrackParseError> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| TrackParseError::DateParseError(date_str.to_string(), e.to_string()))
}

/// Extracts Spotify track ID from a location string
fn extract_spotify_id(location: &str) -> String {
    location
        .split("spotify:track:")
        .nth(1)
        .unwrap_or("")
        .trim_matches(|c| c == '"' || c == ' ')
        .to_string()
}

/// Custom error type for track parsing
#[derive(Debug)]
pub enum TrackParseError {
    FileError(String, String),
    XmlParseError(String),
    DateParseError(String, String),
}

impl fmt::Display for TrackParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrackParseError::FileError(path, e) => write!(f, "Failed to read file '{}': {}", path, e),
            TrackParseError::XmlParseError(e) => write!(f, "XML parse error: {}", e),
            TrackParseError::DateParseError(date, e) => {
                write!(f, "Failed to parse date '{}': {}", date, e)
            }
        }
    }
}

impl std::error::Error for TrackParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_extract_spotify_id() {
        assert_eq!(extract_spotify_id("spotify:track:123abc"), "123abc");
        assert_eq!(extract_spotify_id("file:///music/spotify:track:456def"), "456def");
        assert_eq!(extract_spotify_id("file://localhostspotify:track:7mdDd1a4TtNGqDW1lXc14o\""), "7mdDd1a4TtNGqDW1lXc14o");
        assert_eq!(extract_spotify_id("invalid"), "");
    }

    #[test]
    fn test_parse_date_valid() {
        let date = parse_date("2024-01-15").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_parse_date_invalid() {
        assert!(parse_date("2024/01/15").is_err());
        assert!(parse_date("invalid").is_err());
    }

    #[test]
    fn test_fill_from_xml_empty() {
        let mut tracks = Tracks::new(Vec::new());
        let result = tracks.fill_from_xml("<root></root>", None);
        assert!(result.is_ok());
        assert_eq!(tracks.tracks.len(), 0);
    }
}