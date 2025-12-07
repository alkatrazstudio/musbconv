// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2021, Alexey Parfenov <zxed@alkatrazstudio.net>

use crate::convert::Item;
use crate::cue::find_cue_info;
use lexical_sort::natural_lexical_only_alnum_cmp;
use std::error::Error;
use std::path::Component::{Normal, Prefix};
use std::path::{Component, Path};

pub fn find_files(dirs: &[String], exts: &[String]) -> Result<Vec<Item>, Box<dyn Error>> {
    let mut items = Vec::new();

    for dir in dirs {
        let input_dir = Path::new(dir);
        if !input_dir.exists() {
            return Err(format!("not found: {dir}").into());
        }

        if let Ok(entries) = input_dir.read_dir() {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let path = entry.path();
                    if let Some(filename) = path.to_str() {
                        if file_type.is_dir() {
                            let sub_items = find_files(&[filename.to_owned()], exts)?;
                            items.extend(sub_items);
                        } else if file_type.is_file() {
                            if let Some(ext) = path.extension()
                                && let Some(ext) = ext.to_str()
                            {
                                let ext = ext.to_lowercase();
                                let ext = ext.clone();
                                if !exts.contains(&ext) {
                                    continue;
                                }
                            }
                            if let Some(basename) = path.file_stem()
                                && let Some(basename) = basename.to_str()
                            {
                                let infos = find_cue_info(&path).unwrap_or_default();
                                if infos.is_empty() {
                                    items.push(Item {
                                        filename: filename.to_string(),
                                        basename: basename.to_string(),
                                        index: 0,
                                        total: 0,
                                        cue: None,
                                    });
                                } else {
                                    for info in infos {
                                        items.push(Item {
                                            filename: filename.to_string(),
                                            basename: basename.to_string(),
                                            index: 0,
                                            total: 0,
                                            cue: Some(info),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    items.sort_unstable_by(|a, b| natural_lexical_only_alnum_cmp(&a.basename, &b.basename));
    let n = items.len();
    for (i, item) in items.iter_mut().enumerate() {
        item.index = i;
        item.total = n;
    }

    return Ok(items);
}

fn component_name(component: &Component) -> String {
    return match component {
        Prefix(prefix) => prefix.as_os_str().to_str().unwrap_or_default().to_owned(),
        Normal(s) => s.to_str().unwrap_or_default().to_string(),
        _ => String::default(),
    };
}

pub fn print_tree(base_dir: &str, filenames: &[&String]) {
    println!();
    println!("{base_dir}");

    let mut filenames = filenames.to_vec();
    filenames.sort();
    let base_len = base_dir.len();

    let mut prev_components = Vec::new();
    for filename in filenames {
        let filename = &filename[base_len..];
        let components = Path::new(filename).components().collect::<Vec<Component>>();
        let mut is_diff = false;
        for (i, component) in components.iter().enumerate() {
            let name = component_name(component);
            if !is_diff {
                let prev_name = match prev_components.get(i) {
                    Some(c) => component_name(c),
                    None => String::default(),
                };

                if name == prev_name {
                    continue;
                }

                is_diff = true;
            }

            for _ in 0..i {
                print!("  ");
            }

            println!("{}{}", std::path::MAIN_SEPARATOR, &name);
        }
        prev_components.clone_from(&components);
    }
}
