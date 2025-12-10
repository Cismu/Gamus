use std::collections::HashMap;

/// Claves normalizadas en minúsculas. Deben matchear lo que genera FFmpeg.
pub const KEYS_TITLE: &[&str] = &["title", "tit2", "inam", "\u{a9}nam", "name"];
pub const KEYS_ALBUM: &[&str] = &["album", "talb", "iprd", "\u{a9}alb"];
pub const KEYS_DATE: &[&str] =
  &["date", "year", "original_year", "originalyear", "releasedate", "tdrc", "tyer", "tdor", "\u{a9}day", "icrd"];
pub const KEYS_GENRE: &[&str] = &["genre", "tcon", "ignr", "\u{a9}gen"];
pub const KEYS_TRACK_NUMBER: &[&str] = &["track", "trck", "iprt", "itrk", "trkn"];
pub const KEYS_DISC_NUMBER: &[&str] = &["disc", "tpos", "disk"];

/// Busca el primer valor no vacío asociado a una de las claves proporcionadas.
///
/// Se asume que las claves de `tags` están en minúsculas.
pub fn find_tag_value<'a>(tags: &'a HashMap<String, String>, keys: &[&str]) -> Option<&'a str> {
  keys.iter().find_map(|key| tags.get(*key).map(|v| v.trim())).filter(|v| !v.is_empty())
}

/// Intenta parsear un entero (track, disc, etc.) desde tags que pueden venir como "1/12".
pub fn find_tag_number(tags: &HashMap<String, String>, keys: &[&str]) -> Option<u32> {
  find_tag_value(tags, keys).and_then(|raw| raw.split('/').next()).and_then(|token| token.trim().parse::<u32>().ok())
}
