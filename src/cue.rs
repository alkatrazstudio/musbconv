// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2021, Alexey Parfenov <zxed@alkatrazstudio.net>

use std::path::Path;
use cuna::Cuna;
use cuna::track::Track;
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::str::FromStr;
use std::io::Read;
use std::char::REPLACEMENT_CHARACTER;

const CUE_FRAMES_IN_SECOND: u8 = 75;

pub struct CueInfo {
    pub start: f64,
    pub duration: Option<f64>,
    pub album: String,
    pub title: String,
    pub performer: String,
    pub songwriter: String,
    pub genre: String,
    pub date: String,
    pub disc_id: String,
    pub track: String,
    pub tracks: String,
}

fn read_string_from_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let s = String::from_utf8_lossy(&buf);
    let s = s.replace(REPLACEMENT_CHARACTER, "");
    return Ok(s);
}

fn open_cue(path: &Path) -> Result<Cuna, Box<dyn Error>> {
    let s = read_string_from_file(path)?;
    let rx = Regex::new(r"(?:^|\n)\s*FLAGS.*(?:\n|$)")?;
    let s = rx.replace_all(&s, "\n").to_string();
    let cue = Cuna::from_str(&s)?;
    return Ok(cue);
}

fn find_cue_info_in_file(filename: &Path) -> Option<Vec<CueInfo>> {
    if ! filename.exists() {
        return None;
    }

    match open_cue(&filename) {
        Ok(cue) => {
            if let Some(file) = cue.first_file() {
                let mut infos = Vec::new();
                let max_track_index = max_track_index(&file.tracks);
                for track in &file.tracks {
                    let next_track = track_by_index(&file.tracks, track.id() + 1);
                    if let Some(info) = cue_track_info(&track, next_track, max_track_index, &cue) {
                        infos.push(info);
                    }
                }
                return Some(infos);
            }
        },

        Err(e) => println!("{}", e)
    }

    return None;
}

pub fn find_cue_info(path: &Path) -> Option<Vec<CueInfo>> {
    let cue_filename = path.with_extension("cue");
    if let Some(info) = find_cue_info_in_file(&cue_filename) {
        return Some(info);
    }

    if let Some(cue_filename) = path.to_str() {
        let cue_filename = cue_filename.to_string() + ".cue";
        let cue_filename = Path::new(&cue_filename);
        if let Some(info) = find_cue_info_in_file(cue_filename) {
            return Some(info);
        }
    }

    return None;
}

fn max_track_index(tracks: &[Track]) -> u8 {
    let mut max_id = 0;
    for track in tracks {
        if track.id() > max_id {
            max_id = track.id();
        }
    }
    return max_id;
}

fn track_by_index(tracks: &[Track], id: u8) -> Option<&Track> {
    for track in tracks {
        if track.id() == id {
            return Some(track);
        }
    }
    return None;
}

fn track_start(track: &Track) -> Option<u32> {
    for i in &track.index {
        if i.id() == 1 {
            return Some(i.begin_time.as_frames());
        }
    }
    return None;
}

fn opt_str(s: &[String], def: &str) -> String {
    if let Some(s) = s.first() {
        return s.to_owned();
    }
    return def.into();
}

fn extract_comment(cd: &Cuna, tag: &str) -> String {
    let rx_str = String::from(r"(?i)^") + &regex::escape(tag) + r#"\s+(.+)"?$"#;
    let rx = Regex::new(&rx_str).unwrap();
    for comment in &cd.comments.0 {
        if let Some(m) = rx.captures(&comment) {
            if let Some(m) = m.get(1) {
                let s = m.as_str();
                if s.starts_with('"') && s.ends_with('"') && s.len() > 1 {
                    return s[1..s.len()-1].into();
                }
                return s.into();
            }
        }
    }

    return Default::default();
}

fn cue_track_info(track: &Track, next_track: Option<&Track>, max_track_index: u8, cd: &Cuna) -> Option<CueInfo> {
    if let Some(start) = track_start(track) {
        let mut duration = None;
        if let Some(next_track) = next_track {
            if let Some(next_start) = track_start(next_track) {
                if next_start > start {
                    duration = Some(next_start - start);
                }
            }
        }

        let duration = duration.map(|duration| f64::from(duration) / f64::from(CUE_FRAMES_IN_SECOND));

        return Some(CueInfo {
            start: f64::from(start) / f64::from(CUE_FRAMES_IN_SECOND),
            duration,
            album: opt_str(cd.title(), ""),
            title: opt_str(track.title(), ""),
            performer: opt_str(track.performer(), &opt_str(cd.performer(), "")),
            songwriter: opt_str(track.songwriter(), &opt_str(cd.songwriter(), "")),
            genre: extract_comment(cd, "GENRE"),
            date: extract_comment(cd, "DATE"),
            disc_id: extract_comment(cd, "DISCID"),
            track: track.id().to_string().trim().to_string(),
            tracks: max_track_index.to_string().trim().to_string()
        });
    }
    return None;
}
