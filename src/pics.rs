// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2021, Alexey Parfenov <zxed@alkatrazstudio.net>

use crate::concurrent_map::ConcurrentMap;
use std::process::Command;
use crate::args::AppArgs;
use crate::convert::Progs;
use crate::formats::Format;

pub type PicsMap = ConcurrentMap<String, Option<Vec<u8>>>;

pub fn ffmpeg_conv_pic_args(app_args: &AppArgs) -> Vec<String> {
    let fmt = format!(
        "scale='w=min({},iw)':h='min({},ih)':force_original_aspect_ratio=decrease:flags=lanczos",
        app_args.max_pic_width, app_args.max_pic_height);
    let mut conv_args = vec!["-vf".to_string(), fmt];
    let quality_args = app_args.output_ext_type.pic_quality_args(app_args.pic_quality);
    conv_args.extend(quality_args);
    return conv_args;
}

fn conv_pic(pic_file: &str, app_args: &AppArgs, progs: &Progs) -> Option<Vec<u8>> {
    match app_args.output_ext_type {
        Format::Ogg => return std::fs::read(pic_file).ok(),

        Format::MP3 => {
            let pic_args = ffmpeg_conv_pic_args(app_args);
            let pic_args = pic_args.iter().map(String::as_str).collect::<Vec<&str>>();

            let mut args = vec![
                "-i", pic_file,
                "-f", "mjpeg"
            ];

            args.extend(pic_args);
            args.push("-");

            let args_str = shell_words::join(&args);
            println!("PIC {}: {} {}", pic_file, &progs.ffmpeg_bin, args_str);

            if app_args.dry_run {
                return Some(Vec::new());
            }

            let output = Command::new(&progs.ffmpeg_bin).args(args).output().ok()?;
            if output.status.code()? != 0 {
                println!("PIC {}: {}", pic_file, std::str::from_utf8(&output.stderr).unwrap());
                return None;
            }
            return Some(output.stdout);
        }
    }
}

impl PicsMap {
    pub fn conv_pic_if_needed(&self, pic_filename: &str, args: &AppArgs, progs: &Progs) -> Option<Vec<u8>> {
        if let Some(Some(p)) = self.set_if_not_exists(
            &pic_filename.into(),
            || conv_pic(pic_filename, args, progs)
        ) {
            return Some(p);
        }
        return None;
    }
}

pub fn find_cover_in_dir(dir_name: &str, cover_names: &[String], cover_exts: &[String]) -> Option<String> {
    let entries = std::fs::read_dir(dir_name).ok()?;
    for entry in entries {
        let entry = entry.ok()?;
        if !entry.file_type().ok()?.is_file() {
            continue;
        }

        let ext = entry.path().extension().unwrap_or_default().to_str()?.to_lowercase();
        let basename = entry.path().file_stem()?.to_str()?.to_lowercase();

        for cover_basename in cover_names {
            if basename == *cover_basename {
                for cover_ext in cover_exts {
                    if ext == *cover_ext {
                        let canonical_path = entry.path().canonicalize().ok()?;
                        let file_path = canonical_path.to_str()?;
                        return Some(file_path.into());
                    }
                }
            }
        }
    }

    return None;
}
