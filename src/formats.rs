// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2021, Alexey Parfenov <zxed@alkatrazstudio.net>

pub enum Format {
    MP3,
    Ogg
}

impl Format {
    pub const MIN_QUALITY: u8 = 1;
    pub const MAX_QUALITY: u8 = 100;

    pub fn audio_args(&self) -> Vec<String> {
        return match self {
            Self::MP3 => vec![
                "-b:a", "320k",
                "-write_id3v2", "1",
                "-id3v2_version", "4"
            ],
            Self::Ogg => vec![
                "-b:a", "320k"
            ],
        }.iter().map(|s| (*s).to_string()).collect();
    }

    pub fn normalize_pic_quality(&self, quality: u8) -> u8 {
        let in_range = Self::MAX_QUALITY - Self::MIN_QUALITY;

        let (out_min, out_max) = match self {
            Self::MP3 => (31_i8, 1_i8), // 1 - max quality; 31 - lowest quality
            Self::Ogg => (0_i8, 10_i8) // 0 - lowest quality; 10 - max quality
        };

        let out_range = out_max - out_min;

        let ratio = f32::from(out_range) / f32::from(in_range);
        let out_offset = f32::from(quality) * ratio;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let out_quality = (out_min + out_offset as i8) as u8;
        return out_quality;
    }

    pub fn pic_quality_args(&self, quality: u8) -> Vec<String> {
        let q = self.normalize_pic_quality(quality).to_string();

        return match self {
            Self::MP3 => vec!["-qmin".to_string(), "1".to_string(), "-q:v".to_string(), q],
            Self::Ogg => vec!["-q:v".to_string(), q]
        };
    }
}
