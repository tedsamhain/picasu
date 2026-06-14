use arrayvec::ArrayString;
use bitcode::{Decode, Encode};
use chrono::Utc;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::metadata::AlbumMetadata;
use crate::public::db::tree::TREE;
use crate::public::structure::abstract_data::AbstractData;
use crate::public::structure::object::ObjectSchema;

/// Combined Album data with Object and Metadata
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct AlbumCombined {
    #[serde(flatten)]
    pub object: ObjectSchema,
    #[serde(flatten)]
    pub metadata: AlbumMetadata,
}

/// A helper struct to hold media item info for album calculations
struct MediaItemInfo {
    hash: ArrayString<64>,
    size: u64,
    thumbhash: Option<Vec<u8>>,
    timestamp: i64,
}

impl AlbumCombined {
    pub fn set_cover(&mut self, cover_data: &AbstractData) {
        self.metadata.cover = Some(cover_data.hash());
        self.object.thumbhash = cover_data.thumbhash().cloned();
    }

    fn set_cover_from_info(&mut self, info: &MediaItemInfo) {
        self.metadata.cover = Some(info.hash);
        self.object.thumbhash.clone_from(&info.thumbhash);
    }

    pub fn self_update(&mut self) {
        let ref_data = TREE.in_memory.read().unwrap();

        // Extract fields needed inside the closure to avoid borrowing `self`.
        let album_id = self.object.id;
        let dir_path = self.metadata.dir_path.clone();

        let belongs_to_album = move |alias: &[crate::public::structure::common::FileModify],
                                     albums: &std::collections::HashSet<ArrayString<64>>|
              -> bool {
            if let Some(ref dir) = dir_path {
                // Directory album: membership is path-based — at least one source
                // file must live under the album's directory.
                alias
                    .iter()
                    .any(|a| Path::new(&a.file).starts_with(dir.as_str()))
            } else {
                // Manual album: membership is stored on the media item.
                albums.contains(&album_id)
            }
        };

        let mut data_in_album: Vec<MediaItemInfo> = ref_data
            .par_iter()
            .filter_map(
                |database_timestamp| match &database_timestamp.abstract_data {
                    AbstractData::Image(img) => {
                        if !img.object.is_trashed
                            && belongs_to_album(&img.metadata.alias, &img.metadata.albums)
                        {
                            Some(MediaItemInfo {
                                hash: img.object.id,
                                size: img.metadata.size,
                                thumbhash: img.object.thumbhash.clone(),
                                timestamp: database_timestamp.timestamp,
                            })
                        } else {
                            None
                        }
                    }
                    AbstractData::Video(vid) => {
                        if !vid.object.is_trashed
                            && belongs_to_album(&vid.metadata.alias, &vid.metadata.albums)
                        {
                            Some(MediaItemInfo {
                                hash: vid.object.id,
                                size: vid.metadata.size,
                                thumbhash: vid.object.thumbhash.clone(),
                                timestamp: database_timestamp.timestamp,
                            })
                        } else {
                            None
                        }
                    }
                    AbstractData::Album(_) => None,
                },
            )
            .collect();

        // If there are no items in the album, there's nothing to set
        if data_in_album.is_empty() {
            self.metadata.start_time = None;
            self.metadata.end_time = None;
            self.metadata.cover = None;
            self.object.thumbhash = None;
            self.metadata.item_count = 0;
            self.metadata.item_size = 0;
            return;
        }

        // Sort by timestamp descending (newest first)
        data_in_album.sort_by_key(|info| std::cmp::Reverse(info.timestamp));

        // Set metadata from the sorted list
        self.metadata.start_time = data_in_album.last().map(|info| info.timestamp);
        self.metadata.end_time = data_in_album.first().map(|info| info.timestamp);
        self.metadata.item_count = data_in_album.len();
        self.metadata.item_size = data_in_album.iter().map(|info| info.size).sum();

        // Update last_modified_time
        self.metadata.last_modified_time = Utc::now().timestamp_millis();

        // Set cover if not already set
        if self.metadata.cover.is_none() {
            if let Some(first_info) = data_in_album.first() {
                self.set_cover_from_info(first_info);
            }
        } else {
            // Check if current cover is still in the album, if not update it
            let current_cover = self.metadata.cover.unwrap();
            let cover_still_in_album = data_in_album.iter().any(|info| info.hash == current_cover);
            if !cover_still_in_album && let Some(first_info) = data_in_album.first() {
                self.set_cover_from_info(first_info);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::Path;

    use arrayvec::ArrayString;

    use crate::public::structure::common::FileModify;

    // Mirrors the belongs_to_album closure in self_update.
    fn belongs_to_album(
        alias: &[FileModify],
        albums: &HashSet<ArrayString<64>>,
        dir_path: Option<&str>,
        album_id: ArrayString<64>,
    ) -> bool {
        if let Some(dir) = dir_path {
            alias.iter().any(|a| Path::new(&a.file).starts_with(dir))
        } else {
            albums.contains(&album_id)
        }
    }

    fn alias(paths: &[&str]) -> Vec<FileModify> {
        paths
            .iter()
            .map(|p| FileModify {
                file: p.to_string(),
                modified: 0,
                scan_time: 0,
            })
            .collect()
    }

    #[test]
    fn dir_album_matches_file_inside_dir() {
        let a = alias(&["/photos/vacation/img.jpg"]);
        assert!(belongs_to_album(
            &a,
            &HashSet::new(),
            Some("/photos/vacation"),
            ArrayString::new()
        ));
    }

    #[test]
    fn dir_album_matches_file_in_subdirectory() {
        let a = alias(&["/photos/vacation/day1/img.jpg"]);
        assert!(belongs_to_album(
            &a,
            &HashSet::new(),
            Some("/photos/vacation"),
            ArrayString::new()
        ));
    }

    #[test]
    fn dir_album_does_not_match_sibling_dir() {
        let a = alias(&["/photos/other/img.jpg"]);
        assert!(!belongs_to_album(
            &a,
            &HashSet::new(),
            Some("/photos/vacation"),
            ArrayString::new()
        ));
    }

    #[test]
    fn dir_album_does_not_match_partial_name_prefix() {
        // "/photos/vacation2" must not match dir "/photos/vacation"
        let a = alias(&["/photos/vacation2/img.jpg"]);
        assert!(!belongs_to_album(
            &a,
            &HashSet::new(),
            Some("/photos/vacation"),
            ArrayString::new()
        ));
    }

    #[test]
    fn manual_album_matches_stored_id() {
        let id = ArrayString::from("abc").unwrap();
        let mut albums = HashSet::new();
        albums.insert(id);
        assert!(belongs_to_album(&alias(&[]), &albums, None, id));
    }

    #[test]
    fn manual_album_does_not_match_different_id() {
        let id = ArrayString::from("abc").unwrap();
        let other = ArrayString::from("xyz").unwrap();
        let mut albums = HashSet::new();
        albums.insert(other);
        assert!(!belongs_to_album(&alias(&[]), &albums, None, id));
    }
}
