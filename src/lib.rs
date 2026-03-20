// Re-export modules for testing
pub mod onelibrary;
pub mod spotify_auth;

#[cfg(test)]
mod tests {
    use crate::onelibrary::*;

    #[test]
    fn test_parse_valid_xml_with_spotify_tracks() {
        let xml = r#"
        <LIBRARY>
            <ENTRY Location="file:///music/spotify:track:abc123" DateAdded="2024-01-15">
                <POSITION_MARK/>
            </ENTRY>
        </LIBRARY>
        "#;

        let mut tracks = Tracks::new(Vec::new());
        let result = tracks.fill_from_xml(xml, None);

        assert!(result.is_ok());
        assert_eq!(tracks.tracks.len(), 1);
        assert_eq!(tracks.tracks[0].spotify_id, "abc123");
    }

    #[test]
    fn test_filters_by_date() {
        let xml = r#"
        <LIBRARY>
            <ENTRY Location="spotify:track:old" DateAdded="2023-01-01">
                <POSITION_MARK/>
            </ENTRY>
            <ENTRY Location="spotify:track:new" DateAdded="2024-12-01">
                <POSITION_MARK/>
            </ENTRY>
        </LIBRARY>
        "#;

        let mut tracks = Tracks::new(Vec::new());
        let result = tracks.fill_from_xml(xml, Some("2024-01-01".to_string()));

        assert!(result.is_ok());
        assert_eq!(tracks.tracks.len(), 1);
        assert_eq!(tracks.tracks[0].spotify_id, "new");
    }

    #[test]
    fn test_ignores_non_spotify_tracks() {
        let xml = r#"
        <LIBRARY>
            <ENTRY Location="file:///music/local_track.mp3" DateAdded="2024-01-15">
                <POSITION_MARK/>
            </ENTRY>
            <ENTRY Location="spotify:track:abc123" DateAdded="2024-01-15">
                <POSITION_MARK/>
            </ENTRY>
        </LIBRARY>
        "#;

        let mut tracks = Tracks::new(Vec::new());
        let result = tracks.fill_from_xml(xml, None);

        assert!(result.is_ok());
        assert_eq!(tracks.tracks.len(), 1);
    }

    #[test]
    fn test_ignores_entries_without_position_mark() {
        let xml = r#"
        <LIBRARY>
            <ENTRY Location="spotify:track:abc123" DateAdded="2024-01-15"/>
            <ENTRY Location="spotify:track:def456" DateAdded="2024-01-15">
                <POSITION_MARK/>
            </ENTRY>
        </LIBRARY>
        "#;

        let mut tracks = Tracks::new(Vec::new());
        let result = tracks.fill_from_xml(xml, None);

        assert!(result.is_ok());
        assert_eq!(tracks.tracks.len(), 1);
        assert_eq!(tracks.tracks[0].spotify_id, "def456");
    }
}
