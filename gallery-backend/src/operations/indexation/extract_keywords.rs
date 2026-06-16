use std::collections::HashSet;
use std::path::Path;

/// Extract keyword tags embedded in a file's XMP packet (`dc:subject`), the
/// mechanism most photo tools (Lightroom, digiKam, exiftool `-XMP:Subject`)
/// use to write user-assigned keywords into a file.
///
/// XMP packets are self-delimited, contiguous, plain-UTF8 XML wrapped in
/// `<x:xmpmeta ...> … </x:xmpmeta>` regardless of container format (JPEG
/// `APPn` segment, PNG `iTXt` chunk, raw sidecar, …), so the packet can be
/// located by a plain byte-level substring scan without parsing the
/// container — this function only looks for the `<dc:subject>` element
/// itself, since that's all `extract_keywords_from_file` needs.
///
/// Limitations (see TODO.md "Enable metadata-extraction tests for all
/// supported file formats"): this only handles the common case of an
/// uncompressed, contiguous XMP packet. It will miss keywords stored as
/// compressed PNG `zTXt` text, in an MP4/MOV `uuid` box that splits the
/// packet across multiple boxes, or written via the older binary IPTC IIM
/// mechanism instead of XMP.
pub fn extract_keywords_from_xmp(bytes: &[u8]) -> HashSet<String> {
    const SUBJECT_OPEN: &[u8] = b"<dc:subject>";
    const SUBJECT_CLOSE: &[u8] = b"</dc:subject>";

    let mut keywords = HashSet::new();

    let Some(open_pos) = find_subslice(bytes, SUBJECT_OPEN) else {
        return keywords;
    };
    let content_start = open_pos + SUBJECT_OPEN.len();
    let Some(close_offset) = find_subslice(&bytes[content_start..], SUBJECT_CLOSE) else {
        return keywords;
    };
    let inner = &bytes[content_start..content_start + close_offset];
    let Ok(inner_text) = std::str::from_utf8(inner) else {
        return keywords;
    };

    // Pull out every <rdf:li>...</rdf:li> entry inside the dc:subject
    // element (normally wrapped in an rdf:Bag, but we don't care which
    // container tag it's in — just the leaf items).
    let mut rest = inner_text;
    while let Some(li_start) = rest.find("<rdf:li") {
        let from_li = &rest[li_start..];
        let Some(tag_end) = from_li.find('>') else {
            break;
        };
        let content = &from_li[tag_end + 1..];
        let Some(li_end) = content.find("</rdf:li>") else {
            break;
        };
        let keyword = content[..li_end].trim();
        if !keyword.is_empty() {
            keywords.insert(keyword.to_string());
        }
        rest = &content[li_end + "</rdf:li>".len()..];
    }

    keywords
}

/// Find the first occurrence of `needle` in `haystack`, treating both as
/// raw bytes (not UTF-8) since `haystack` may be a whole binary file.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
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
