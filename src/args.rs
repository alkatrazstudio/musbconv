// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2021, Alexey Parfenov <zxed@alkatrazstudio.net>

use clap::{App, Arg, ErrorKind};
use chrono::{DateTime};
use std::error::Error;
use std::io::BufWriter;
use std::process::exit;

pub struct AppArgs {
    pub input_dirs: Vec<String>,
    pub output_dir: String,
    pub filename_template: String,
    pub dry_run: bool,
    pub input_exts: Vec<String>,
    pub output_ext: String,
    pub overwrite: bool,
    pub ffmpeg_opts: Vec<String>,
    pub max_pic_width: u16,
    pub max_pic_height: u16,
    pub pic_quality: u8,
    pub use_embed_pic: bool,
    pub ffmpeg_bin: Option<String>,
    pub ffprobe_bin: Option<String>,
    pub threads_count: usize,
    pub cover_names: Vec<String>,
    pub cover_exts: Vec<String>,
    pub min_track_number_digits: u8
}

fn validate_num_func(min: i64, max: i64) -> impl Fn(String) -> Result<(), String> {
    return move |v| {
        return match v.parse::<i64>() {
            Ok(i) => {
                if i < min || i > max {
                    return Err(format!("The number must be between {} and {}, but got {}.", min, max, i));
                }
                return Ok(());
            }
            Err(_) => Err(format!("\"{}\" can't be parsed as a number.", v))
        };
    };
}

fn opt_string_vec(opt: Option<&str>) -> Vec<String> {
    let parts = opt.unwrap_or_default();
    let parts = parts.split(',').map(|part| part.to_lowercase().trim().to_string()).collect::<Vec<String>>();
    return parts;
}

pub fn parse_cli_args() -> Result<Option<AppArgs>, Box<dyn Error>> {
    let v = "v".to_owned() + env!("CARGO_PKG_VERSION");
    let ts = DateTime::parse_from_rfc3339(env!("VERGEN_BUILD_TIMESTAMP"))?;
    let day = ts.format("%e").to_string().trim_start().to_string();
    let ts_str = ts.format(format!("%B {}, %Y", &day).as_str()).to_string();
    let git_hash = &env!("VERGEN_GIT_SHA");
    let about = format!("\n\
            Performs a batch conversion between audio formats using ffmpeg.\n\
            Uses multiple threads if possible.\n\
            Supports CUE sheets and album art.\n\
            \n\
            Project homepage: https://github.com/alkatrazstudio/musbconv\n\
            License: GPLv3\n\
            Build date: {}\n\
            Git commit: {}\n\
            Author: Alexey Parfenov (a.k.a. ZXED) <zxed@alkatrazstudio.net>\n\
            Author homepage: https://alkatrazstudio.net",
                &ts_str, git_hash);

    let mut app = App::new("musbconv")
        .long_about(about.as_str())
        .version(v.as_str())

        .arg(Arg::with_name("INPUT_DIR")
            .long("input-dir")
            .long_help("\
                Directory to search for audio files.\n\
                This directory will be searched recursively.\n\
                Only files with INPUT_EXT extensions will be considered.\n\
                This option can be specified multiple times.\n")
            .required(true)
            .multiple(true)
            .number_of_values(1)
            .empty_values(false)
            .display_order(0))

        .arg(Arg::with_name("OUTPUT_DIR")
            .long("output-dir")
            .long_help("\
                Base directory for writing the converted files.\n\
                All converted output files will be located under this directory.\n\
                The actual location of the file depends on a FILENAME_TEMPLATE.\n")
            .required(true)
            .empty_values(false)
            .display_order(1))

        .arg(Arg::with_name("FILENAME_TEMPLATE")
            .long("filename-template")
            .long_help("\
                Template for the output filename inside OUTPUT_DIR.\n\
                The template is in Handlebars format (https://handlebarsjs.com).\n\
                Supported placeholders:\n\
                \x20 {{title}} - track title (if empty: defaults to {{file_base}})\n\
                \x20 {{album}} - album name (if empty: defaults to {{dir_name}})\n\
                \x20 {{artist}} - artist (if empty: defaults to {{author}} or {{performer}})\n\
                \x20 {{catalog_number}} - catalog number\n\
                \x20 {{author}} - track author (if empty: defaults to {{artist}} or {{performer}})\n\
                \x20 {{comment}} - comment\n\
                \x20 {{composer}} - composer (if empty: defaults to {{songwriter}}, {{lyricist}} or {{artist}})\n\
                \x20 {{lyricist}} - lyricist (if empty: defaults to {{songwriter}}, {{composer}} or {{artist}})\n\
                \x20 {{songwriter}} - songwriter (if empty: defaults to {{composer}}, {{lyricist}} or {{artist}})\n\
                \x20 {{date}} - track/album date ((if empty: defaults to {{date}})\n\
                \x20 {{disc}} - disc number\n\
                \x20 {{discs}} - total number of discs\n\
                \x20 {{disc_id}} - disc ID\n\
                \x20 {{track}} - track number (can be taken from the track itself or its cue-sheet)\n\
                \x20 {{tracks}} - number of tracks in the album (can be taken from the track itself or its cue-sheet)\n\
                \x20 {{genre}} - genre\n\
                \x20 {{label}} - music label\n\
                \x20 {{performer}} - performer\n\
                \x20 {{publisher}} - publisher\n\
                \x20 {{year}} - year (if empty: defaults to {{date}} if it starts with 4 digits)\n\
                \x20 {{file_name}} - input file name with the extension, but without the directory path\n\
                \x20 {{dir_name}} - directory name (without parent directories)\n\
                \x20 {{file_base}} - input file name without the extension\n\
                \x20 {{file_ext}} - file extension without a leading dot\n\
                All values in these placeholders will be present, but some of them may be empty strings.\n\
                The values will be sanitized for a safe usage in a file paths\n\
                and also directory separators will be removed.\n")
            .required(true)
            .empty_values(false)
            .display_order(2))

        .arg(Arg::with_name("FFMPEG_OPTIONS")
            .long_help("\
                Additional ffmpeg options.\n\
                It's better to specify these options after a \"--\".\n\
                Example: musbconv ... -- -b:a 128k\n")
            .multiple(true))

        .arg(Arg::with_name("DRY_RUN")
            .long("dry-run")
            .long_help("\
                Dry-run.\n\
                Do not write anything to the disk,\n\
                just list the files that will be generated.\n")
            .possible_values(&["y", "n"])
            .value_name("y|n")
            .default_value("n"))

        .arg(Arg::with_name("INPUT_EXT")
            .long("input-ext")
            .long_help( "\
                Comma-separated list of file extensions to search for.\n\
                Only the files with these extensions will be converted.\n\
                The list is case-insensitive.\n\
                Not all output formats may be supported by ffmpeg.\n\
                Run \"ffmpeg -formats\" to show a list of the supported formats\n\
                (search for \"D\"-formats).\n")
            .default_value("flac,wv,m4a")
            .empty_values(false)
            .value_name("ext1,ext2,..."))

        .arg(Arg::with_name("OUTPUT_EXT")
            .long("output-ext")
            .long_help( "\
                Extension for the output filename.\n\
                The extension also defines the format (e.g. mp3, ogg).\n\
                Some formats have predefined ffmpeg settings:\n\
                - MP3: -b:a 320k -write_id3v2 1 -id3v2_version 4\n\
                - OGG: -b:a 320k\n\
                The extension/format name is case-insensitive.\n\
                Not all output formats may be supported by ffmpeg.\n\
                Run \"ffmpeg -formats\" to show a list of the supported formats\n\
                (search for \"E\"-formats).\n")
            .default_value("mp3")
            .empty_values(false)
            .value_name("ext"))

        .arg(Arg::with_name("OVERWRITE")
            .long("overwrite")
            .long_help("\
                Overwrite existing files.\n\
                y - overwrite the file if it already exists.\n\
                n - if the output file already exists then count it as an error.\n")
            .possible_values(&["y", "n"])
            .value_name("y|n")
            .default_value("n"))

        .arg(Arg::with_name("MAX_PIC_WIDTH")
            .long("max-pic-width")
            .long_help("\
                Maximum width for a cover art in pixels.\n\
                The aspect ratio of the cover art will be preserved.\n\
                Must be in range of 1-5000.\n")
            .value_name("WIDTH")
            .default_value("500")
            .validator(validate_num_func(1, 5000)))

        .arg(Arg::with_name("MAX_PIC_HEIGHT")
            .long("max-pic-height")
            .long_help("\
                Maximum height for a cover art in pixels.\n\
                The aspect ratio of the cover art will be preserved.\n\
                Must be in range of 1-5000.\n")
            .value_name("HEIGHT")
            .default_value("500")
            .validator(validate_num_func(1, 5000)))

        .arg(Arg::with_name("PIC_QUALITY")
            .long("pic-quality")
            .long_help("\
                Quality for a cover art.\n\
                Only applies when the cover art is bigger than the allowed dimensions\n\
                and needs to be re-encoded.\n\
                1 - max quality\n\
                31 - lowest quality\n")
            .value_name("QUALITY")
            .default_value("2")
            .validator(validate_num_func(1, 31)))

        .arg(Arg::with_name("USE_EMBED_PIC")
            .long("use-embed-pic")
            .long_help("\
                Use and/or preserve the embedded album art if possible.\n\
                If set to \"n\" then the embedded album art will be always ignored,\n\
                and external images will be used instead.\n")
            .possible_values(&["y", "n"])
            .value_name("y|n")
            .default_value("y"))

        .arg(Arg::with_name("COVER_NAME")
            .long("cover-name")
            .long_help("\
                Comma-separated list of file names of cover art images.\n\
                The file names do not include extensions (which are specified via --cover-ext).\n\
                The file names are case-insensitive.\n\
                Set to an empty string (--cover-name=\"\") to disable loading cover art from files.\n")
            .value_name("FILENAME")
            .default_value("folder,cover,album,albumartsmall,thumb,front,scan"))

        .arg(Arg::with_name("COVER_EXT")
            .long("cover-ext")
            .long_help("\
                Comma-separated list of file extensions of cover art images.\n\
                The leading dot must not be specified.\n\
                The file extensions are case-insensitive.\n\
                You can specify an empty string as one of the extensions to load cover art from files without extensions.\n\
                An empty string (--cover-ext=\"\") will mean load only files without extensions.\n\
                Only formats supported by ffmpeg will be processed.\n")
            .value_name("FILENAME")
            .default_value("jpeg,jpg,png,gif"))

        .arg(Arg::with_name("MIN_TRACK_NUMBER_DIGITS")
            .long("min-track-number-digits")
            .long_help("\
                Minimum number of digits for a resulting track number string.\n\
                This affects {{track}} and {{tracks}} placeholders in FILENAME_TEMPLATE but not the file tags.\n\
                The track number will be padded with zeroes if the number of digits is less than specified.\n\
                The resulting number of digits will be picked as maximum between\n\
                --min-track-number-digits and the original number of digits in {{tracks}}.\n\
                Examples:\n\
                \x20 a) if the original file has {{track}}=\"42\" and {{tracks}}=50,\n\
                \x20    then --min-track-number-digits=3 will make {{track}}=\"042\" and {{tracks}}=050.\n\
                \x20 b) if the original file has {{track}}=\"13\" and {{tracks}}=150,\n\
                \x20    then --min-track-number-digits=1 will make {{track}}=\"013\" and {{tracks}}=150.\n\
                Must be in range of 1-10.\n")
            .value_name("MIN_DIGITS")
            .default_value("2")
            .validator(validate_num_func(1, 10)))

        .arg(Arg::with_name("FFMPEG_BIN")
            .long("ffmpeg-bin")
            .long_help("\
                Path for ffmpeg program.\n\
                If not specified then ffmpeg is searched in PATH.\n")
            .value_name("PATH_TO_FFMPEG_BINARY"))

        .arg(Arg::with_name("FFPROBE_BIN")
            .long("ffprobe-bin")
            .long_help("\
                Path for ffprobe program.\n\
                If not specified then ffprobe is searched in PATH.\n")
            .value_name("PATH_TO_FFPROBE_BINARY"))

        .arg(Arg::with_name("THREADS")
            .long("threads")
            .long_help("\
                Number of threads to simultaneously run ffmpeg in.\n\
                Must be between 0 and 1024.\n\
                If not specified or zero then the number of threads is chosen automatically.\n")
            .default_value("0")
            .validator(validate_num_func(0, 1024)))

        .after_help(
            "EXAMPLES:\n\
            \n\
            \x20    a) Simple. Put the converted files into folders named after the artists and the albums.\n\
            \n\
            \x20      musbconv --input-dir=/home/user/Music/flac --output-dir=/home/user/Music/mp3 \\\n\
            \x20        --filename-template=\"{{artist}}/{{year}} - {{album}}/{{track}}. {{title}}\"\n\
            \n\
            \x20    b) Complex. Specify some custom musbconv options, ffmpeg options,\n\
            \x20         process files from multiple directories, \n\
            \x20         use a template with conditional statements and perform a dry-run.\n\
            \n\
            \x20      musbconv --input-dir=flac_folder1 --input-dir=flac_folder2 --output-dir=ogg_folder \\\n\
            \x20        --filename-template=\"{{artist}}/{{year}} - {{album}}/{{#if disc}}CD {{disc}}/{{/if}}{{track}}. {{title}}\" \\\n\
            \x20        --input-ext=flac,wv --output-ext=ogg --overwrite=y --dry-run=y \\\n\
            \x20        --max-pic-width=256 --max-pic-height=256 --pic_quality=5  \\\n\
            \x20        -- -b:a 128k\n\
            \n\
            \x20    c) For Windows. Specify custom path for ffmpeg and ffprobe.\n\
            \x20         Note: on Windows you need to use either \\\\ or / as a directory separator inside the FILENAME_TEMPLATE.
            \n\
            \x20      musbconv.exe --ffmpeg-bin=C:\\Downloads\\ffmpeg\\bin\\ffmpeg.exe --ffprobe-bin=C:\\Downloads\\ffmpeg\\bin\\ffprobe.exe \\\n\
            \x20        --input-dir=\"C:\\Users\\user\\Music\\flac music\" --output-dir=\"C:\\Users\\user\\Music\\mp3 music\" \\\n\
            \x20        --filename-template=\"{{artist}}\\\\{{year}} - {{album}}\\\\{{track}}. {{title}}\"");

    let mut buf = BufWriter::new(Vec::new());
    app.write_long_help(&mut buf)?;
    let help_bytes = buf.into_inner()?;
    let help_str = String::from_utf8(help_bytes)?;

    let matches = app.get_matches_safe();

    match matches {
        Ok(matches) => {
            let input_exts = opt_string_vec(matches.value_of("INPUT_EXT"));
            let cover_names = opt_string_vec(matches.value_of("COVER_NAME"));
            let cover_exts = opt_string_vec(matches.value_of("COVER_EXT"));

            let ffmpeg_opts = matches.values_of("FFMPEG_OPTIONS").unwrap_or_default().map(|s| s.to_string()).collect();

            return Ok(Some(AppArgs {
                input_dirs: matches.values_of("INPUT_DIR").unwrap().map(|s| s.to_owned()).collect(),
                output_dir: matches.value_of("OUTPUT_DIR").unwrap().to_string(),
                filename_template: matches.value_of("FILENAME_TEMPLATE").unwrap().to_string(),
                dry_run: matches.value_of("DRY_RUN").unwrap() == "y",
                input_exts,
                output_ext: matches.value_of("OUTPUT_EXT").unwrap().to_lowercase(),
                overwrite: matches.value_of("OVERWRITE").unwrap() == "y",
                ffmpeg_opts,
                max_pic_height: matches.value_of("MAX_PIC_HEIGHT").unwrap().parse::<u16>()?,
                max_pic_width: matches.value_of("MAX_PIC_WIDTH").unwrap().parse::<u16>()?,
                pic_quality: matches.value_of("PIC_QUALITY").unwrap().parse::<u8>()?,
                use_embed_pic: matches.value_of("USE_EMBED_PIC").unwrap() == "y",
                ffmpeg_bin: matches.value_of("FFMPEG_BIN").map(|s| s.to_string()),
                ffprobe_bin: matches.value_of("FFPROBE_BIN").map(|s| s.to_string()),
                threads_count: matches.value_of("THREADS").unwrap().parse::<usize>()?,
                cover_names,
                cover_exts,
                min_track_number_digits: matches.value_of("MIN_TRACK_NUMBER_DIGITS").unwrap().parse::<u8>()?,
            }));
        }
        Err(e) => match e.kind {
            ErrorKind::HelpDisplayed => {
                println!("{}", &help_str);
                return Ok(None);
            },
            ErrorKind::VersionDisplayed => {
                println!();
                return Ok(None);
            }
            _ => {
                println!("{}", e.message);
                exit(1);
            }
        }
    }
}
