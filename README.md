# musbconv

Performs a batch conversion between audio formats using ffmpeg.
Uses multiple threads if possible.
Supports CUE sheets and album art.


## Usage

```
USAGE:
    musbconv [OPTIONS] --filename-template <FILENAME_TEMPLATE> --input-dir <INPUT_DIR>... --output-dir <OUTPUT_DIR> [--] [FFMPEG_OPTIONS]...

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information


OPTIONS:
        --input-dir <INPUT_DIR>...
            Directory to search for audio files.
            This directory will be searched recursively.
            Only files with INPUT_EXT extensions will be considered.
            This option can be specified multiple times.
        --output-dir <OUTPUT_DIR>
            Base directory for writing the converted files.
            All converted output files will be located under this directory.
            The actual location of the file depends on a FILENAME_TEMPLATE.
        --filename-template <FILENAME_TEMPLATE>
            Template for the output filename inside OUTPUT_DIR.
            The template is in Handlebars format (https://handlebarsjs.com).
            Supported placeholders:
              {{title}} - track title (if empty: defaults to {{file_base}})
              {{album}} - album name (if empty: defaults to {{dir_name}})
              {{artist}} - artist (if empty: defaults to {{author}} or {{performer}})
              {{catalog_number}} - catalog number
              {{author}} - track author (if empty: defaults to {{artist}} or {{performer}})
              {{comment}} - comment
              {{composer}} - composer (if empty: defaults to {{songwriter}}, {{lyricist}} or {{artist}})
              {{lyricist}} - lyricist (if empty: defaults to {{songwriter}}, {{composer}} or {{artist}})
              {{songwriter}} - songwriter (if empty: defaults to {{composer}}, {{lyricist}} or {{artist}})
              {{date}} - track/album date ((if empty: defaults to {{date}})
              {{disc}} - disc number
              {{discs}} - total number of discs
              {{disc_id}} - disc ID
              {{track}} - track number (can be taken from the track itself or its cue-sheet)
              {{tracks}} - number of tracks in the album (can be taken from the track itself or its cue-sheet)
              {{genre}} - genre
              {{label}} - music label
              {{performer}} - performer
              {{publisher}} - publisher
              {{year}} - year (if empty: defaults to {{date}} if it starts with 4 digits)
              {{file_name}} - input file name with the extension, but without the directory path
              {{dir_name}} - directory name (without parent directories)
              {{file_base}} - input file name without the extension
              {{file_ext}} - file extension without a leading dot
            All values in these placeholders will be present, but some of them may be empty strings.
            The values will be sanitized for a safe usage in a file paths
            and also directory separators will be removed.
        --cover-ext <FILENAME>
            Comma-separated list of file extensions of cover art images.
            The leading dot must not be specified.
            The file extensions are case-insensitive.
            You can specify an empty string as one of the extensions to load cover art from files without extensions.
            An empty string (--cover-ext="") will mean load only files without extensions.
            Only formats supported by ffmpeg will be processed.
             [default: jpeg,jpg,png,gif]
        --cover-name <FILENAME>
            Comma-separated list of file names of cover art images.
            The file names do not include extensions (which are specified via --cover-ext).
            The file names are case-insensitive.
            Set to an empty string (--cover-name="") to disable loading cover art from files.
             [default: folder,cover,album,albumartsmall,thumb,front,scan]
        --dry-run <y|n>
            Dry-run.
            Do not write anything to the disk,
            just list the files that will be generated.
             [default: n]  [possible values: y, n]
        --ffmpeg-bin <PATH_TO_FFMPEG_BINARY>
            Path for ffmpeg program.
            If not specified then ffmpeg is searched in PATH.
        --ffprobe-bin <PATH_TO_FFPROBE_BINARY>
            Path for ffprobe program.
            If not specified then ffprobe is searched in PATH.
        --input-ext <ext1,ext2,...>
            Comma-separated list of file extensions to search for.
            Only the files with these extensions will be converted.
            The list is case-insensitive.
            Not all output formats may be supported by ffmpeg.
            Run "ffmpeg -formats" to show a list of the supported formats
            (search for "D"-formats).
             [default: flac,wv,m4a]
        --max-pic-height <HEIGHT>
            Maximum height for a cover art in pixels.
            The aspect ratio of the cover art will be preserved.
            Must be in range of 1-5000.
             [default: 500]
        --max-pic-width <WIDTH>
            Maximum width for a cover art in pixels.
            The aspect ratio of the cover art will be preserved.
            Must be in range of 1-5000.
             [default: 500]
        --min-track-number-digits <MIN_DIGITS>
            Minimum number of digits for a resulting track number string.
            This affects {{track}} and {{tracks}} placeholders in FILENAME_TEMPLATE but not the file tags.
            The track number will be padded with zeroes if the number of digits is less than specified.
            The resulting number of digits will be picked as maximum between
            --min-track-number-digits and the original number of digits in {{tracks}}.
            Examples:
              a) if the original file has {{track}}="42" and {{tracks}}=50,
                 then --min-track-number-digits=3 will make {{track}}="042" and {{tracks}}=050.
              b) if the original file has {{track}}="13" and {{tracks}}=150,
                 then --min-track-number-digits=1 will make {{track}}="013" and {{tracks}}=150.
            Must be in range of 1-10.
             [default: 2]
        --output-ext <ext>
            Extension for the output filename.
            The extension also defines the format (e.g. mp3, ogg).
            Some formats have predefined ffmpeg settings:
            - MP3: -b:a 320k -write_id3v2 1 -id3v2_version 4
            - OGG: -b:a 320k
            The extension/format name is case-insensitive.
            Not all output formats may be supported by ffmpeg.
            Run "ffmpeg -formats" to show a list of the supported formats
            (search for "E"-formats).
             [default: mp3]
        --overwrite <y|n>
            Overwrite existing files.
            y - overwrite the file if it already exists.
            n - if the output file already exists then count it as an error.
             [default: n]  [possible values: y, n]
        --pic-quality <QUALITY>
            Quality for a cover art.
            Only applies when the cover art is bigger than the allowed dimensions
            and needs to be re-encoded.
            1 - max quality
            31 - lowest quality
             [default: 2]
        --threads <THREADS>
            Number of threads to simultaneously run ffmpeg in.
            Must be between 0 and 1024.
            If not specified or zero then the number of threads is chosen automatically.
             [default: 0]
        --use-embed-pic <y|n>
            Use and/or preserve the embedded album art if possible.
            If set to "n" then the embedded album art will be always ignored,
            and external images will be used instead.
             [default: y]  [possible values: y, n]

ARGS:
    <FFMPEG_OPTIONS>...
            Additional ffmpeg options.
            It's better to specify these options after a "--".
            Example: musbconv ... -- -b:a 128k

EXAMPLES:

     a) Simple. Put the converted files into folders named after the artists and the albums.

       musbconv --input-dir=/home/user/Music/flac --output-dir=/home/user/Music/mp3 \
         --filename-template="{{artist}}/{{year}} - {{album}}/{{track}}. {{title}}"

     b) Complex. Specify some custom musbconv options, ffmpeg options,
          process files from multiple directories,
          use a template with conditional statements and perform a dry-run.

       musbconv --input-dir=flac_folder1 --input-dir=flac_folder2 --output-dir=ogg_folder \
         --filename-template="{{artist}}/{{year}} - {{album}}/{{#if disc}}CD {{disc}}/{{/if}}{{track}}. {{title}}" \
         --input-ext=flac,wv --output-ext=ogg --overwrite=y --dry-run=y \
         --max-pic-width=256 --max-pic-height=256 --pic_quality=5  \
         -- -b:a 128k

     c) For Windows. Specify custom path for ffmpeg and ffprobe.
          Note: on Windows you need to use either \\ or / as a directory separator inside the FILENAME_TEMPLATE.

       musbconv.exe --ffmpeg-bin=C:\Downloads\ffmpeg\bin\ffmpeg.exe --ffprobe-bin=C:\Downloads\ffmpeg\bin\ffprobe.exe \
         --input-dir="C:\Users\user\Music\flac music" --output-dir="C:\Users\user\Music\mp3 music" \
         --filename-template="{{artist}}\\{{year}} - {{album}}\\{{track}}. {{title}}"
```


## Minimum system requirements

- Ubuntu 20.04 (x86_64)
- Windows 10 version 1909 (x86_64)
- macOS 10.15 Catalina (x86_64)

Also, you need `ffmpeg` and `ffprobe` installed.


## License

GPLv3
