# musbconv - CHANGELOG


## v0.5.0 (August 15, 2021)

- Changed: TOTALDISCS=1 is now ignored when constructing the filename


## v0.4.0 (July 17, 2021)

- Added: support for new CUE tags: DISCNUMBER and TOTALDISCS
- Fixed: tags contain leading and trailing whitespace


## v0.3.0 (June 6, 2021)

- Added: APE as one of default input formats
- Added: find CUE files that include the music file extension in their filename
- Fixed: cannot convert to OGG with an external cover art
- Fixed: track number detection can't parse X/Y format
- Fixed: CUE "performer" is not treated as "artist"
- Removed: support for any output format except MP3 and OGG


## v0.2.0 (May 16, 2021)

- Fixed: ffmpeg errors are not captured when a cover art is piped


## v0.1.0 (May 9, 2021)

- Initial release
