// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2021, Alexey Parfenov <zxed@alkatrazstudio.net>

use crate::args::{AppArgs, parse_cli_args};
use crate::convert::{Item, Progs, conv_item, validate_template};
use crate::files::{find_files, print_tree};
use crate::pics::PicsMap;
use rayon::prelude::*;
use std::error::Error;
use std::path::Path;

enum ItemResult {
    Filename(String),
    Error(String),
}

fn run(items: &[Item], args: &AppArgs, progs: &Progs) -> Result<Vec<ItemResult>, Box<dyn Error>> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads_count)
        .build_global()?;

    let pics = PicsMap::new();
    let filenames = items
        .par_iter()
        .map(|item| {
            return match conv_item(item, &pics, args, progs) {
                Err(e) => {
                    item.print_info("ERR", &e.to_string());
                    return ItemResult::Error(e.to_string());
                }
                Ok(filename) => ItemResult::Filename(filename),
            };
        })
        .collect();

    return Ok(filenames);
}

fn find_prog(name: &str, arg: &Option<String>) -> Result<String, Box<dyn Error>> {
    if let Some(a) = arg.clone() {
        let path = Path::new(&a);
        if !path.exists() {
            return Err(format!(
                "{} does not exists.",
                path.to_str().ok_or("Can't convert path to string")?
            )
            .into());
        }
        return Ok(path
            .to_str()
            .ok_or("Can't convert path to string")?
            .to_string());
    }
    return match which::which(name) {
        Ok(file_path) => Ok(file_path
            .to_str()
            .ok_or("Can't convert path to string")?
            .to_string()),
        Err(_) => Err(format!("{} is not found.", &name).into()),
    };
}

fn find_progs(args: &AppArgs) -> Result<Progs, Box<dyn Error>> {
    return Ok(Progs {
        ffmpeg_bin: find_prog("ffmpeg", &args.ffmpeg_bin)?,
        ffprobe_bin: find_prog("ffprobe", &args.ffprobe_bin)?,
    });
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_cli_args()?;
    if let Some(args) = args {
        validate_template(&args.filename_template)?;
        let progs = find_progs(&args)?;

        let items = find_files(&args.input_dirs, &args.input_exts)?;
        let filenames = run(&items, &args, &progs)?;
        let mut valid_filenames = Vec::new();
        let mut errs = Vec::new();

        let n = filenames.len();

        for a in 0..n {
            let filename = &filenames[a];
            match filename {
                ItemResult::Filename(filename) => {
                    let mut exists = false;
                    for b in (a + 1)..n {
                        if let ItemResult::Filename(other_filename) = &filenames[b] {
                            if filename.eq(other_filename) {
                                exists = true;
                                errs.push(format!(
                                    "{}: resolves to {} just as {}",
                                    &items[a].filename, &filename, &items[b].filename
                                ));
                                break;
                            }
                        }
                    }

                    if !exists {
                        valid_filenames.push(filename);
                    }
                }
                ItemResult::Error(e) => errs.push(format!("{}: {}", &items[a].filename, e)),
            }
        }

        if !errs.is_empty() {
            println!();
            println!("ERRORS OCCURRED:");
            for err in &errs {
                println!("{}", &err);
            }
        }

        if !valid_filenames.is_empty() {
            print_tree(&args.output_dir, &valid_filenames);
        }

        println!();
        if args.dry_run {
            println!("DRY-RUN!");
        }
        println!("Converted files: {}", valid_filenames.len());
        println!("Errors occurred: {}", errs.len());

        if errs.is_empty() {
            return Ok(());
        }

        println!();
        return Err("Some errors occurred".into());
    }
    return Ok(());
}
