use std::collections::HashMap;

pub const KEYS_TITLE: &[&str] = &["title", "tit2", "inam", "\u{a9}nam", "name"];

pub const KEYS_ALBUM: &[&str] = &["album", "talb", "iprd", "\u{a9}alb"];

pub const KEYS_ARTIST_TRACK: &[&str] = &["artist", "tpe1", "iart", "\u{a9}art", "auth"];

// Nota: FFmpeg a veces normaliza esto a "album_artist"
pub const KEYS_ARTIST_ALBUM: &[&str] = &["album_artist", "album artist", "albumartist", "tpe2", "aart"];

pub const KEYS_DATE: &[&str] =
  &["date", "year", "original_year", "originalyear", "releasedate", "tdrc", "tyer", "tdor", "\u{a9}day", "icrd"];

pub const KEYS_GENRE: &[&str] = &["genre", "tcon", "ignr", "\u{a9}gen"];

pub const KEYS_TRACK_NUMBER: &[&str] = &["track", "trck", "iprt", "itrk", "trkn"];

pub const KEYS_DISC_NUMBER: &[&str] = &["disc", "tpos", "disk"];

/// Busca el primer valor no vacío asociado a una de las claves proporcionadas.
pub fn find_tag_value(tags: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
  for key in keys {
    if let Some(val) = tags.get(&String::from(*key)) {
      let trimmed = val.trim();
      if !trimmed.is_empty() {
        return Some(trimmed.to_string());
      }
    }
  }
  None
}
