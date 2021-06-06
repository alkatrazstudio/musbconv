// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2021, Alexey Parfenov <zxed@alkatrazstudio.net>

use std::process::Command;
use std::collections::HashMap;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use regex::{Regex};
use lazy_static::lazy_static;
use std::path::Path;
use std::ffi::OsStr;
use std::error::Error;
use sanitize_filename::{sanitize_with_options, Options};
use crate::cue::CueInfo;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize)]
pub struct MetaStreamTags {
    comment: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct MetaStream {
    codec_type: String,
    width: Option<u32>,
    height: Option<u32>,
    tags: Option<MetaStreamTags>
}

impl MetaStreamTags {
    const COVER: &'static str = "Cover (front)";
}

impl MetaStream {
    const VIDEO: &'static str = "video";
}

#[derive(Serialize, Deserialize)]
pub struct MetaFormat {
    tags: Option<HashMap<String, Value>>
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    streams: Vec<MetaStream>,
    format: MetaFormat
}

#[derive(Serialize, Default, Clone)]
pub struct MetaTags {
    pub title: String,
    pub album: String,
    pub artist: String,
    pub catalog_number: String,
    pub author: String,
    pub comment: String,
    pub composer: String,
    pub lyricist: String,
    pub songwriter: String,
    pub date: String,
    pub disc: String,
    pub discs: String,
    pub disc_id: String,
    pub track: String,
    pub tracks: String,
    pub genre: String,
    pub label: String,
    pub performer: String,
    pub publisher: String,
    pub year: String,
    pub file_name: String,
    pub dir_name: String,
    pub file_base: String,
    pub file_ext: String
}

#[derive(Default)]
pub struct FileMeta {
    pub has_pic: bool,
    pub pic_width: u32,
    pub pic_height: u32,
    pub tags: MetaTags
}

fn to_str(x: Option<&OsStr>) -> String {
    return x.unwrap_or_default().to_str().unwrap().to_string();
}

fn first_val(map: &HashMap<String, &String>, keys: &[&str]) -> String {
    for key in keys {
        if let Some(v) = map.get(*key) {
            return v.to_string();
        }
    }
    return Default::default();
}

fn fill_tags(hash: &HashMap<String, Value>, filename: &str, cue: &Option<CueInfo>) -> MetaTags {
    lazy_static! {
        static ref RX_ALPHA: Regex = Regex::new(r"[^a-z]").unwrap();
        static ref RX_TRACK: Regex = Regex::new(r"^(\d+)/(\d+)$").unwrap();
    }

    let mut tags = HashMap::new();

    let file_path = Path::new(filename).canonicalize().unwrap();
    let dir_path = file_path.parent().unwrap();

    let mut keys = hash.keys().into_iter().collect::<Vec<_>>();
    keys.sort_by(|a, b| {
        let ord = a.to_lowercase().cmp(&b.to_lowercase());
        if ord == Ordering::Equal {
            return a.cmp(b);
        }
        return ord;
    });

    for key in keys {
        let tag_key = key.to_lowercase();
        let tag_key = RX_ALPHA.replace_all(&tag_key, "").to_string();
        if let Value::String(val) = &hash[key] {
            tags.insert(tag_key, val);
        }
    }

    let mut meta_tags = MetaTags {
        title: first_val(&tags, &["title"]),
        album: first_val(&tags, &["album"]),
        artist: first_val(&tags, &["albumartist", "artist", "artists"]),
        catalog_number: first_val(&tags, &["catalog", "catalognumber"]),
        author: first_val(&tags, &["author"]),
        comment: first_val(&tags, &["comment"]),
        composer: first_val(&tags, &["composer"]),
        lyricist: first_val(&tags, &["lyricist"]),
        songwriter: first_val(&tags, &["songwriter"]),
        date: first_val(&tags, &["date", "originaldate", "originalreleasedate"]),
        disc: first_val(&tags, &["disc"]),
        discs: first_val(&tags, &["disctotal", "totaldiscs"]),
        disc_id: first_val(&tags, &["discid"]),
        track: first_val(&tags, &["track"]),
        tracks: first_val(&tags, &["tracktotal", "totaltracks"]),
        genre: first_val(&tags, &["genre"]),
        label: first_val(&tags, &["label"]),
        performer: first_val(&tags, &["performer"]),
        publisher: first_val(&tags, &["publisher"]),
        year: first_val(&tags, &["year"]),
        file_name: to_str(file_path.file_name()),
        dir_name: to_str(dir_path.file_name()),
        file_base: to_str(file_path.file_stem()),
        file_ext: to_str(file_path.extension())
    };

    if let Some(cue) = cue {
        if !cue.album.is_empty() {
            meta_tags.album = cue.album.clone();
        }
        if !cue.title.is_empty() {
            meta_tags.title = cue.title.clone();
        }
        if !cue.songwriter.is_empty() {
            meta_tags.songwriter = cue.songwriter.clone();
        }
        if !cue.genre.is_empty() {
            meta_tags.genre = cue.genre.clone();
        }
        if !cue.performer.is_empty() {
            meta_tags.performer = cue.performer.clone();
            meta_tags.artist = cue.performer.clone();
        }
        if !cue.date.is_empty() {
            meta_tags.date = cue.date.clone();
        }
        if !cue.disc_id.is_empty() {
            meta_tags.disc_id = cue.disc_id.clone();
        }
        if !cue.track.is_empty() {
            meta_tags.track = cue.track.clone();
        }
        if !cue.tracks.is_empty() {
            meta_tags.tracks = cue.tracks.clone();
        }
    }

    if let Some(m) = RX_TRACK.captures(&meta_tags.track) {
        if let (Some(m1), Some(m2)) = (m.get(1), m.get(2)) {
            if meta_tags.tracks.is_empty() {
                meta_tags.tracks = m2.as_str().to_string();
            }
            meta_tags.track = m1.as_str().to_string();
        }
    }

    return meta_tags;
}

fn filesafe_str(s: &str) -> String {
    return sanitize_with_options(s, Options {
        replacement: "",
        windows: false,
        truncate: true
    });
}

pub fn sanitize_tags(meta: &MetaTags) -> MetaTags {
    return MetaTags {
        title: filesafe_str(&meta.title),
        album: filesafe_str(&meta.album),
        artist: filesafe_str(&meta.artist),
        catalog_number: filesafe_str(&meta.catalog_number),
        author: filesafe_str(&meta.author),
        comment: filesafe_str(&meta.comment),
        composer: filesafe_str(&meta.composer),
        lyricist: filesafe_str(&meta.lyricist),
        songwriter: filesafe_str(&meta.songwriter),
        date: filesafe_str(&meta.date),
        disc: filesafe_str(&meta.disc),
        discs: filesafe_str(&meta.discs),
        disc_id: filesafe_str(&meta.disc_id),
        track: filesafe_str(&meta.track),
        tracks: filesafe_str(&meta.tracks),
        genre: filesafe_str(&meta.genre),
        label: filesafe_str(&meta.label),
        performer: filesafe_str(&meta.performer),
        publisher: filesafe_str(&meta.publisher),
        year: filesafe_str(&meta.year),
        file_name: filesafe_str(&meta.file_name),
        dir_name: filesafe_str(&meta.dir_name),
        file_base: filesafe_str(&meta.file_base),
        file_ext: filesafe_str(&meta.file_ext)
    }
}

pub fn fill_fallback_tags(meta_tags: &MetaTags) -> MetaTags {
    let mut meta_tags = meta_tags.clone();

    if meta_tags.year.is_empty() && !meta_tags.date.is_empty() {
        lazy_static! {
            static ref RX: Regex = Regex::new(r"^\d{4}").unwrap();
        }
        if let Some(m) = RX.find(&meta_tags.date) {
            meta_tags.year = m.as_str().into();
        }
    } else if !meta_tags.year.is_empty() && meta_tags.date.is_empty() {
        meta_tags.date = meta_tags.year.clone();
    }

    if meta_tags.title.is_empty() {
        meta_tags.title = meta_tags.file_base.clone();
    }

    if meta_tags.album.is_empty() {
        meta_tags.album = meta_tags.dir_name.clone();
    }

    if meta_tags.artist.is_empty() {
        if !meta_tags.author.is_empty() {
            meta_tags.artist = meta_tags.author.clone();
        } else if !meta_tags.performer.is_empty() {
            meta_tags.artist = meta_tags.performer.clone();
        }
    }

    if meta_tags.author.is_empty() && !meta_tags.artist.is_empty() {
        meta_tags.author = meta_tags.artist.clone();
    }

    if meta_tags.songwriter.is_empty() {
        if !meta_tags.composer.is_empty() {
            meta_tags.songwriter = meta_tags.composer.clone();
        } else if !meta_tags.lyricist.is_empty() {
            meta_tags.songwriter = meta_tags.lyricist.clone();
        } else if !meta_tags.artist.is_empty() {
            meta_tags.songwriter = meta_tags.artist.clone();
        }
    }

    if meta_tags.composer.is_empty() && !meta_tags.songwriter.is_empty() {
        meta_tags.composer = meta_tags.songwriter.clone();
    }

    if meta_tags.lyricist.is_empty() && !meta_tags.songwriter.is_empty() {
        meta_tags.lyricist = meta_tags.songwriter.clone();
    }

    return meta_tags;
}

pub fn extract_meta(filename: &str, cue: &Option<CueInfo>, ffprobe_bin: &str) -> Result<FileMeta, Box<dyn Error>> {
    let out = Command::new(ffprobe_bin)
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            filename
        ])
        .output()?.stdout;
    let out = std::str::from_utf8(&out)?;
    let meta: Meta = serde_json::from_str(&out)?;

    let format_tags = meta.format.tags.unwrap_or_default();
    let tags = fill_tags(&format_tags, filename, cue);

    let mut fmeta = FileMeta {
        tags,
        ..Default::default()
    };

    for s in meta.streams {
        if s.codec_type == MetaStream::VIDEO {
            if let Some(tags) = s.tags {
                if let Some(comment) = tags.comment {
                    if comment == MetaStreamTags::COVER {
                        if let Some(w) = s.width {
                            if let Some(h) = s.height {
                                fmeta.has_pic = true;
                                fmeta.pic_height = h;
                                fmeta.pic_width = w;
                            }
                        }
                    }
                }
            }
        }
    }

    return Ok(fmeta);
}
