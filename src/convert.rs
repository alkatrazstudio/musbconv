// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2021, Alexey Parfenov <zxed@alkatrazstudio.net>

use crate::cue::CueInfo;
use crate::pics::{PicsMap, find_cover_in_dir, ffmpeg_conv_pic_args};
use crate::args::AppArgs;
use std::error::Error;
use std::path::Path;
use crate::meta::{extract_meta, fill_fallback_tags, sanitize_tags, MetaTags};
use handlebars::Handlebars;
use std::process::Command;
use std::io::Write;
use path_dedot::ParseDot;
use crate::Progs;
use std::cmp::max;
use crate::formats::Format;

pub struct Item {
    pub filename: String,
    pub basename: String,
    pub index: usize,
    pub total: usize,
    pub cue: Option<CueInfo>
}

impl Item {
    pub fn print_info(&self, cat: &str, info: &str) {
        println!("[{}/{}:{}] {}", self.index + 1, self.total, cat, info);
    }

    fn print_args(&self, cmd: &str, args: &[String]) {
        let args = shell_words::join(args);
        self.print_info("CMD", &format!("{} {}", cmd, args));
    }
}

fn sanitize_filename(filename: &str) -> Result<String, Box<dyn Error>> {
    let path = String::from("/..///") + filename;
    let path = Path::new(&path).parse_dot()?;
    let path = path.strip_prefix("/")?;
    let path = path.to_str().ok_or("Can't convert path to string")?;
    return Ok(path.to_string());
}

macro_rules! str_vec {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

fn add_meta(args: &mut Vec<std::string::String>, val: &str, name: &str) {
    if !val.is_empty() {
        args.extend(str_vec![
            "-metadata", &format!("{}={}", name, val)
        ]);
    }
}

fn render_template(template: &str, tags: &MetaTags) -> Result<String, Box<dyn Error>> {
    let mut hb = Handlebars::new();
    hb.set_strict_mode(true);
    hb.register_escape_fn(|s| s.into());

    let result = hb.render_template(template, tags)?;
    return Ok(result);
}

pub fn validate_template(template: &str) -> Result<(), Box<dyn Error>> {
    let tags = MetaTags {
        ..Default::default()
    };
    if let Err(e) = render_template(template, &tags) {
        return Err(format!("{}", e).into());
    }

    let tags = MetaTags {
        title: "1".to_string(),
        album: "1".to_string(),
        artist: "1".to_string(),
        catalog_number: "1".to_string(),
        author: "1".to_string(),
        comment: "1".to_string(),
        composer: "1".to_string(),
        lyricist: "1".to_string(),
        songwriter: "1".to_string(),
        date: "1".to_string(),
        disc: "1".to_string(),
        discs: "1".to_string(),
        disc_id: "1".to_string(),
        track: "1".to_string(),
        tracks: "1".to_string(),
        genre: "1".to_string(),
        label: "1".to_string(),
        performer: "1".to_string(),
        publisher: "1".to_string(),
        year: "1".to_string(),
        file_name: "1".to_string(),
        dir_name: "1".to_string(),
        file_base: "1".to_string(),
        file_ext: "1".to_string()
    };
    if let Err(e) = render_template(template, &tags) {
        return Err(format!("{}", e).into());
    }

    return Ok(());
}

pub fn conv_item(item: &Item, pics: &PicsMap, app_args: &AppArgs, progs: &Progs) -> Result<String, Box<dyn Error>>
{
    let input_filename = &item.filename;
    item.print_info("INFO", &format!("processing {}", &input_filename));
    let canonical_path = Path::new(input_filename).parent()
        .ok_or(format!("no parent for {}", input_filename))?.canonicalize()?;
    let input_dir = canonical_path.to_str().ok_or("Can't get a string from the canonical path")?;

    let meta = extract_meta(input_filename, &item.cue, &progs.ffprobe_bin)?;
    let mut tags = fill_fallback_tags(&meta.tags);
    if !tags.tracks.is_empty() {
        tags.tracks = format!("{:0>width$}", tags.tracks, width = app_args.min_track_number_digits as usize);
    }
    if !tags.track.is_empty() {
        let tracks_digits_count = max(tags.tracks.len(), app_args.min_track_number_digits as usize);
        tags.track = format!("{:0>width$}", tags.track, width = tracks_digits_count);
    }

    let filename_tags = sanitize_tags(&tags);

    let filename = render_template(&app_args.filename_template, &filename_tags)?;
    let filename = filename + "." + &app_args.output_ext;
    let output_filename = sanitize_filename(&filename)?;

    let output_path = Path::new(&app_args.output_dir).join(&output_filename);
    let output_path_str = output_path.to_str().ok_or("Can't convert path to string")?;
    let dir_path = output_path.parent().ok_or(format!("no parent for {}", output_path_str))?;

    if !app_args.overwrite && output_path.exists() {
        return Err(format!("file exists: {}", output_path_str).into());
    }

    if !app_args.dry_run {
        std::fs::create_dir_all(dir_path)?;
    }

    let mut args = str_vec![
        "-hide_banner", "-nostats",
        "-loglevel", "warning",
        "-y"
    ];

    let mut audio_args = app_args.output_ext_type.audio_args();

    add_meta(&mut audio_args, &meta.tags.album, "album");
    add_meta(&mut audio_args, &meta.tags.composer, "composer");
    add_meta(&mut audio_args, &meta.tags.genre, "genre");
    add_meta(&mut audio_args, &meta.tags.title, "title");
    add_meta(&mut audio_args, &meta.tags.artist, "artist");
    add_meta(&mut audio_args, &meta.tags.performer, "performer");
    add_meta(&mut audio_args, &meta.tags.disc, "disc");
    add_meta(&mut audio_args, &meta.tags.publisher, "publisher");
    add_meta(&mut audio_args, &meta.tags.date, "date");
    add_meta(&mut audio_args, &meta.tags.year, "year");

    if !meta.tags.track.is_empty() && !meta.tags.tracks.is_empty() {
        add_meta(&mut audio_args, &(meta.tags.track + "/" + &meta.tags.tracks), "track");
    } else {
        add_meta(&mut audio_args, &meta.tags.track, "track");
    }

    let start_str;
    let duration_str;
    if let Some(cue) = &item.cue {
        start_str = format!("{:.3}", cue.start);
        args.extend(str_vec![
            "-ss:a", &start_str
        ]);

        duration_str = if let Some(duration) = cue.duration {
            format!("{:.3}", duration)
        } else {
            String::default()
        };

        if !duration_str.is_empty() {
            args.extend(str_vec![
                "-t:a", &duration_str
            ]);
        }
    }
    let output;

    args.extend(str_vec![
        "-i", &input_filename
    ]);
    if meta.has_pic && app_args.use_embed_pic {
        args.extend(audio_args);

        let pic_args = ffmpeg_conv_pic_args(app_args);
        args.extend(pic_args);
        let mut args = args.iter().chain(&app_args.ffmpeg_opts).cloned().collect::<Vec<String>>();
        args.push(output_path_str.to_string());

        item.print_args(&progs.ffmpeg_bin, &args);
        if app_args.dry_run {
            output = None;
        } else {
            output = Some(Command::new(&progs.ffmpeg_bin).args(args).output()?);
        }
    } else {
        let output_pic_data;
        if let Some(input_pic_filename) = find_cover_in_dir(input_dir, &app_args.cover_names, &app_args.cover_exts) {
            output_pic_data = pics.conv_pic_if_needed(&input_pic_filename, &app_args, &progs);
            if output_pic_data == None {
                return Err(format!("can't convert: {}", &input_pic_filename).into());
            }
        } else {
            output_pic_data = None;
        }

        if let Some(output_pic_data) = output_pic_data {
            args.extend(str_vec![
                "-i", "-"
            ]);
            args.extend(audio_args);
            args.extend(str_vec![
                "-map", "0:a", "-map", "1:v",
                "-metadata:s:v", "title=Album cover", "-metadata:s:v", "comment=Cover (front)"
            ]);
            match app_args.output_ext_type {
                Format::MP3 => {
                    args.extend(str_vec!["-c:v", "copy"]);
                },
                Format::Ogg => {
                    args.extend(str_vec!["-c:v", "libtheora"]);
                    let pic_conv_args = ffmpeg_conv_pic_args(&app_args);
                    args.extend(pic_conv_args);
                }
            }
            let mut args = args.iter().chain(&app_args.ffmpeg_opts).cloned().collect::<Vec<String>>();
            args.push(output_path_str.into());

            item.print_args(&progs.ffmpeg_bin, &args);
            if app_args.dry_run {
                output = None;
            } else {
                let mut proc = Command::new(&progs.ffmpeg_bin).args(&args)
                    .stdin(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()?;
                if let Some(stdin) = proc.stdin.as_mut() {
                    stdin.write_all(&output_pic_data)?;
                    stdin.flush()?;
                }
                output = Some(proc.wait_with_output()?);
            }
        } else {
            args.extend(audio_args);
            let mut args = args.iter().chain(&app_args.ffmpeg_opts).cloned().collect::<Vec<String>>();
            args.push(output_path_str.into());

            item.print_args(&progs.ffmpeg_bin, &args);
            if app_args.dry_run {
                output = None;
            } else {
                output = Some(Command::new(&progs.ffmpeg_bin).args(args).output()?);
            }
        }
    }

    if let Some(output) = output {
        if output.status.code().ok_or("Cannot get the exit code")? != 0 {
            return Err(std::str::from_utf8(&output.stderr)?.into());
        }
    }

    return Ok(output_path_str.into());
}
