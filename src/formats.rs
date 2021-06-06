pub enum Format {
    MP3,
    OGG
}

impl Format {
    pub const MIN_QUALITY: u8 = 1;
    pub const MAX_QUALITY: u8 = 100;

    pub fn audio_args(self: &Self) -> Vec<String> {
        return match self {
            Format::MP3 => vec![
                "-b:a", "320k",
                "-write_id3v2", "1",
                "-id3v2_version", "4"
            ],
            Format::OGG => vec![
                "-b:a", "320k"
            ],
        }.iter().map(|s| s.to_string()).collect();
    }

    pub fn normalize_pic_quality(self: &Self, quality: u8) -> u8 {
        let in_range = Self::MAX_QUALITY - Self::MIN_QUALITY;

        let (out_min, out_max) = match self {
            Format::MP3 => (31 as u8, 1 as u8), // 1 - max quality; 31 - lowest quality
            Format::OGG => (0 as u8, 10 as u8) // 0 - lowest quality; 10 - max quality
        };

        let out_range = out_max as i8 - out_min as i8;

        let ratio = out_range as f32 / in_range as f32;
        let out_offset = quality as f32 * ratio;
        let out_quality = (out_min as i8 + out_offset as i8) as u8;
        return out_quality;
    }

    pub fn pic_quality_args(self: &Self, quality: u8) -> Vec<String> {
        let q = self.normalize_pic_quality(quality).to_string();

        return match self {
            Format::MP3 => vec!["-qmin".to_string(), "1".to_string(), "-q:v".to_string(), q],
            Format::OGG => vec!["-q:v".to_string(), q]
        };
    }
}
