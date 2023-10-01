// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2021, Alexey Parfenov <zxed@alkatrazstudio.net>

use clap::{Arg, ArgAction, Command};
use std::error::Error;
use std::io::BufWriter;
use std::process::exit;
use clap::builder::{NonEmptyStringValueParser, RangedU64ValueParser};
use clap::error::ErrorKind;
use crate::formats::Format;

mod built {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub struct AppArgs {
    pub input_dirs: Vec<String>,
    pub output_dir: String,
    pub filename_template: String,
    pub dry_run: bool,
    pub input_exts: Vec<String>,
    pub output_ext: String,
    pub output_ext_type: Format,
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

fn opt_string_vec(opt: Option<&String>) -> Vec<String> {
    let parts = if let Some(opt) = opt {
         opt.split(',').map(|part| part.to_lowercase().trim().to_string()).collect::<Vec<String>>()
    } else {
        Vec::new()
    };
    return parts;
}

pub fn parse_cli_args() -> Result<Option<AppArgs>, Box<dyn Error>> {
    let v = "v".to_owned() + built::PKG_VERSION;
    let git_hash = built::GIT_COMMIT_HASH.unwrap_or_default();
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
            built::BUILT_TIME_UTC, git_hash);

    let pic_quality_help = format!("\
        Quality for a cover art.\n\
        Only applies when the cover art is bigger than the allowed dimensions\n\
        and needs to be re-encoded.\n\
        {} - lowest quality\n\
        {} - highest quality",
            Format::MIN_QUALITY, Format::MAX_QUALITY);

    let mp3_audio_args = Format::audio_args(&Format::MP3).join(" ");
    let ogg_audio_args = Format::audio_args(&Format::Ogg).join(" ");
    let output_ext_help = format!("\
        Extension/format for the output filename.\n\
        The formats have predefined ffmpeg settings:\n\
        * mp3: {}\n\
        * ogg: {}",
            mp3_audio_args, ogg_audio_args);

    let mut app = Command::new("musbconv")
        .long_about(about)
        .version(v)

        .arg(Arg::new("INPUT_DIR")
            .long("input-dir")
            .long_help("\
                Directory to search for audio files.\n\
                This directory will be searched recursively.\n\
                Only files with INPUT_EXT extensions will be considered.\n\
                This option can be specified multiple times.")
            .required(true)
            .action(ArgAction::Append)
            .number_of_values(1)
            .value_parser(NonEmptyStringValueParser::new())
            .display_order(0))

        .arg(Arg::new("OUTPUT_DIR")
            .long("output-dir")
            .long_help("\
                Base directory for writing the converted files.\n\
                All converted output files will be located under this directory.\n\
                The actual location of the file depends on a FILENAME_TEMPLATE.")
            .required(true)
            .value_parser(NonEmptyStringValueParser::new())
            .display_order(1))

        .arg(Arg::new("FILENAME_TEMPLATE")
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
                and also directory separators will be removed.")
            .required(true)
            .value_parser(NonEmptyStringValueParser::new())
            .display_order(2))

        .arg(Arg::new("FFMPEG_OPTIONS")
            .long_help("\
                Additional ffmpeg options.\n\
                It's better to specify these options after a \"--\".\n\
                Example: musbconv ... -- -b:a 128k")
            .action(ArgAction::Append))

        .arg(Arg::new("DRY_RUN")
            .long("dry-run")
            .long_help("\
                Dry-run.\n\
                Do not write anything to the disk,\n\
                just list the files that will be generated.")
            .value_parser(["y", "n"])
            .value_name("y|n")
            .default_value("n"))

        .arg(Arg::new("INPUT_EXT")
            .long("input-ext")
            .long_help( "\
                Comma-separated list of file extensions to search for.\n\
                Only the files with these extensions will be converted.\n\
                The list is case-insensitive.\n\
                Not all output formats may be supported by ffmpeg.\n\
                Run \"ffmpeg -formats\" to show a list of the supported formats\n\
                (search for \"D\"-formats).")
            .default_value("flac,wv,m4a,ape")
            .value_parser(NonEmptyStringValueParser::new())
            .value_name("ext1,ext2,..."))

        .arg(Arg::new("OUTPUT_EXT")
            .long("output-ext")
            .long_help(&output_ext_help)
            .default_value("mp3")
            .value_parser(["mp3", "ogg"])
            .value_parser(NonEmptyStringValueParser::new())
            .value_name("ext"))

        .arg(Arg::new("OVERWRITE")
            .long("overwrite")
            .long_help("\
                Overwrite existing files.\n\
                y - overwrite the file if it already exists.\n\
                n - if the output file already exists then count it as an error.")
            .value_parser(["y", "n"])
            .value_name("y|n")
            .default_value("n"))

        .arg(Arg::new("MAX_PIC_WIDTH")
            .long("max-pic-width")
            .long_help("\
                Maximum width for a cover art in pixels.\n\
                The aspect ratio of the cover art will be preserved.\n\
                Must be in range of 1-5000.")
            .value_name("WIDTH")
            .default_value("500")
            .value_parser(RangedU64ValueParser::<u16>::new().range(1..5000)))

        .arg(Arg::new("MAX_PIC_HEIGHT")
            .long("max-pic-height")
            .long_help("\
                Maximum height for a cover art in pixels.\n\
                The aspect ratio of the cover art will be preserved.\n\
                Must be in range of 1-5000.")
            .value_name("HEIGHT")
            .default_value("500")
            .value_parser(RangedU64ValueParser::<u16>::new().range(1..5000)))

        .arg(Arg::new("PIC_QUALITY")
            .long("pic-quality")
            .long_help(&pic_quality_help)
            .value_name("QUALITY")
            .default_value("96")
            .value_parser(RangedU64ValueParser::<u8>::new().range(1..5000)))

        .arg(Arg::new("USE_EMBED_PIC")
            .long("use-embed-pic")
            .long_help("\
                Use and/or preserve the embedded album art if possible.\n\
                If set to \"n\" then the embedded album art will be always ignored,\n\
                and external images will be used instead.")
            .value_parser(["y", "n"])
            .value_name("y|n")
            .default_value("y"))

        .arg(Arg::new("COVER_NAME")
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
                Only formats supported by ffmpeg will be processed.")
            .value_name("FILENAME")
            .default_value("jpeg,jpg,png,gif"))

        .arg(Arg::new("MIN_TRACK_NUMBER_DIGITS")
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
                Must be in range of 1-10.")
            .value_name("MIN_DIGITS")
            .default_value("2")
            .value_parser(RangedU64ValueParser::<u8>::new().range(1..10)))

        .arg(Arg::new("FFMPEG_BIN")
            .long("ffmpeg-bin")
            .long_help("\
                Path for ffmpeg program.\n\
                If not specified then ffmpeg is searched in PATH.")
            .value_name("PATH_TO_FFMPEG_BINARY"))

        .arg(Arg::new("FFPROBE_BIN")
            .long("ffprobe-bin")
            .long_help("\
                Path for ffprobe program.\n\
                If not specified then ffprobe is searched in PATH.")
            .value_name("PATH_TO_FFPROBE_BINARY"))

        .arg(Arg::new("THREADS")
            .long("threads")
            .long_help("\
                Number of threads to simultaneously run ffmpeg in.\n\
                Must be between 0 and 1024.\n\
                If not specified or zero then the number of threads is chosen automatically.")
            .default_value("0")
            .value_parser(RangedU64ValueParser::<usize>::new().range(0..1024)))

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
            \x20        --max-pic-width=256 --max-pic-height=256 --pic_quality=50 \\\n\
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

    let matches = app.try_get_matches();

    match matches {
        Ok(matches) => {
            let input_exts = opt_string_vec(matches.get_one("INPUT_EXT"));
            let cover_names = opt_string_vec(matches.get_one("COVER_NAME"));
            let cover_exts = opt_string_vec(matches.get_one("COVER_EXT"));

            let ffmpeg_opts = matches.get_many("FFMPEG_OPTIONS").unwrap_or_default().map(|s: &String| s.clone()).collect();

            let output_ext: &String = matches.get_one("OUTPUT_EXT").unwrap();
            let output_ext_type = match output_ext.as_str() {
                "mp3" => Format::MP3,
                "ogg" => Format::Ogg,
                _ => return Err(format!("Unsupported extension: {}", output_ext).into())
            };

            return Ok(Some(AppArgs {
                input_dirs: matches.get_many("INPUT_DIR").unwrap().map(|s: &String| s.to_owned()).collect(),
                output_dir: matches.get_one::<String>("OUTPUT_DIR").unwrap().clone(),
                filename_template: matches.get_one::<String>("FILENAME_TEMPLATE").unwrap().clone(),
                dry_run: matches.get_one::<String>("DRY_RUN").unwrap().as_str() == "y",
                input_exts,
                output_ext: output_ext.clone(),
                output_ext_type,
                overwrite: matches.get_one::<String>("OVERWRITE").unwrap().as_str() == "y",
                ffmpeg_opts,
                max_pic_height: *matches.get_one::<u16>("MAX_PIC_HEIGHT").unwrap(),
                max_pic_width: *matches.get_one::<u16>("MAX_PIC_WIDTH").unwrap(),
                pic_quality: *matches.get_one::<u8>("PIC_QUALITY").unwrap(),
                use_embed_pic: matches.get_one::<String>("USE_EMBED_PIC").unwrap().as_str() == "y",
                ffmpeg_bin: matches.get_one::<String>("FFMPEG_BIN").map(|s| s.clone()),
                ffprobe_bin: matches.get_one::<String>("FFPROBE_BIN").map(|s| s.clone()),
                threads_count: *matches.get_one::<usize>("THREADS").unwrap(),
                cover_names,
                cover_exts,
                min_track_number_digits: *matches.get_one::<u8>("MIN_TRACK_NUMBER_DIGITS").unwrap(),
            }));
        }
        Err(e) => match e.kind() {
            ErrorKind::DisplayHelp => {
                println!("{}", &help_str);
                return Ok(None);
            },
            ErrorKind::DisplayVersion => {
                println!("{}", built::PKG_VERSION);
                return Ok(None);
            }
            _ => {
                println!("{}", e);
                exit(1);
            }
        }
    }
}
