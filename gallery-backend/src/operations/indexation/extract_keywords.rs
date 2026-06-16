use std::collections::HashSet;
use std::path::Path;

/// Extract keyword tags embedded in a file's XMP packet (`dc:subject`), the
/// mechanism most photo tools (Lightroom, digiKam, exiftool `-XMP:Subject`)
/// use to write user-assigned keywords into a file.
///
/// XMP packets are self-delimited, contiguous, plain-UTF8 XML wrapped in
/// `<x:xmpmeta ...> … </x:xmpmeta>` regardless of container format (JPEG
/// `APPn` segment, PNG `iTXt` chunk, raw sidecar, …), so a packet can be
/// located by a plain substring scan of the raw bytes without parsing the
/// container.
///
/// NOT YET IMPLEMENTED — always returns an empty set. Tracked as a known
/// gap in TODO.md ("tags discovered at index time"); the e2e scenarios in
/// `tests/e2e.rs` (`scenario_n_*`) and the unit tests below define the
/// contract this function must satisfy once implemented.
pub fn extract_keywords_from_xmp(bytes: &[u8]) -> HashSet<String> {
    let _ = bytes;
    HashSet::new()
}

/// Convenience wrapper that reads `path` and delegates to
/// [`extract_keywords_from_xmp`]. Returns an empty set (instead of an
/// error) if the file cannot be read, since keyword extraction is a
/// best-effort enrichment step and must never fail indexing.
pub fn extract_keywords_from_file(path: &Path) -> HashSet<String> {
    match std::fs::read(path) {
        Ok(bytes) => extract_keywords_from_xmp(&bytes),
        Err(_) => HashSet::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal XMP packet embedding the given `dc:subject` keywords,
    /// matching what exiftool/Lightroom/digiKam write.
    fn xmp_packet_with_keywords(keywords: &[&str]) -> String {
        let items: String = keywords
            .iter()
            .map(|k| format!("<rdf:li>{k}</rdf:li>"))
            .collect();
        format!(
            r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
<rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/">
<dc:subject><rdf:Bag>{items}</rdf:Bag></dc:subject>
</rdf:Description>
</rdf:RDF>
</x:xmpmeta>"#
        )
    }

    #[test]
    fn extracts_keywords_from_dc_subject_bag() {
        let xmp = xmp_packet_with_keywords(&["family", "vacation"]);
        let keywords = extract_keywords_from_xmp(xmp.as_bytes());
        assert_eq!(
            keywords,
            HashSet::from(["family".to_string(), "vacation".to_string()])
        );
    }

    #[test]
    fn finds_packet_embedded_inside_arbitrary_container_bytes() {
        // Simulates a real file: binary JPEG segments before and after the
        // XMP packet. The scan must locate the packet regardless of
        // surrounding bytes.
        let xmp = xmp_packet_with_keywords(&["sunset"]);
        let mut bytes = b"\xff\xd8\xff\xe0JFIF garbage binary prefix".to_vec();
        bytes.extend_from_slice(xmp.as_bytes());
        bytes.extend_from_slice(b"more binary jpeg scan data\xff\xd9");

        let keywords = extract_keywords_from_xmp(&bytes);
        assert_eq!(keywords, HashSet::from(["sunset".to_string()]));
    }

    #[test]
    fn returns_empty_set_when_no_xmp_packet_present() {
        let keywords = extract_keywords_from_xmp(b"\xff\xd8\xff plain jpeg, no xmp");
        assert!(keywords.is_empty());
    }

    #[test]
    fn returns_empty_set_when_dc_subject_is_absent_or_empty() {
        let xmp = xmp_packet_with_keywords(&[]);
        let keywords = extract_keywords_from_xmp(xmp.as_bytes());
        assert!(keywords.is_empty());
    }
}
