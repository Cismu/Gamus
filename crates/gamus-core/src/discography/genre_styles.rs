use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Representa los géneros musicales principales utilizados dentro del sistema.
///
/// Este listado está inspirado en la taxonomía de Discogs y refleja categorías
/// amplias de clasificación musical. Se usa en metadatos importados desde archivos,
/// scrapers o bases de datos externas.
///
/// *Nota:* El valor no captura subgéneros; para eso existe [`Style`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Genre {
  Rock,
  Electronic,
  Pop,
  FolkWorldAndCountry,
  Jazz,
  FunkSoul,
  Classical,
  HipHop,
  Latin,
  StageAndScreen,
  Reggae,
  Blues,
  NonMusic,
  Childrens,
  BrassAndMilitary,
}

impl fmt::Display for Genre {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let text = match self {
      Genre::Rock => "Rock",
      Genre::Electronic => "Electronic",
      Genre::Pop => "Pop",
      Genre::FolkWorldAndCountry => "Folk, World, & Country",
      Genre::Jazz => "Jazz",
      Genre::FunkSoul => "Funk / Soul",
      Genre::Classical => "Classical",
      Genre::HipHop => "Hip Hop",
      Genre::Latin => "Latin",
      Genre::StageAndScreen => "Stage & Screen",
      Genre::Reggae => "Reggae",
      Genre::Blues => "Blues",
      Genre::NonMusic => "Non-Music",
      Genre::Childrens => "Children's",
      Genre::BrassAndMilitary => "Brass & Military",
    };
    write!(f, "{}", text)
  }
}

/// Error producido cuando una cadena no puede convertirse en [`Genre`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid genre: {input}")]
pub struct GenreParseError {
  pub input: String,
}

impl FromStr for Genre {
  type Err = GenreParseError;

  /// Intenta convertir una cadena en un [`Genre`].
  ///
  /// Normaliza la cadena eliminando espacios, guiones y separadores comunes.
  /// Si la cadena no coincide con ningún género conocido, se devuelve un error.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let normalized = s.trim().to_lowercase().replace(['-', ' ', ',', '&', '/'], "");

    let genre = match normalized.as_str() {
      "rock" => Genre::Rock,
      "electronic" => Genre::Electronic,
      "pop" => Genre::Pop,
      "folkworldandcountry" | "folkworldcountry" => Genre::FolkWorldAndCountry,
      "jazz" => Genre::Jazz,
      "funksoul" => Genre::FunkSoul,
      "classical" => Genre::Classical,
      "hiphop" => Genre::HipHop,
      "latin" => Genre::Latin,
      "stageandscreen" | "stagescreen" => Genre::StageAndScreen,
      "reggae" => Genre::Reggae,
      "blues" => Genre::Blues,
      "nonmusic" => Genre::NonMusic,
      "childrens" | "children" => Genre::Childrens,
      "brassandmilitary" | "brassmilitary" => Genre::BrassAndMilitary,
      _ => return Err(GenreParseError { input: s.to_string() }),
    };

    Ok(genre)
  }
}

/// Describe estilos musicales más específicos que un [`Genre`].
///
/// Los estilos representan subgéneros, movimientos o etiquetas de escena.
/// Por ejemplo: *Synth-pop*, *Hardcore*, *Ambient*, *J-pop*, *Vocaloid*, etc.
///
/// También incluye un caso genérico [`Style::Custom`] para permitir almacenar
/// variantes no contempladas explícitamente, preservando el valor original.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Style {
  // --- Orden basado en popularidad aproximada de Discogs ---
  PopRock,
  House,
  Vocal,
  Experimental,
  Punk,
  AlternativeRock,
  SynthPop,
  Techno,
  IndieRock,
  Ambient,
  Soul,
  Disco,
  Hardcore,
  Folk,
  Ballad,
  Country,
  HardRock,
  Electro,
  RockAndRoll,
  Chanson,
  Romantic,
  Trance,
  HeavyMetal,
  PsychedelicRock,
  FolkRock,

  // --- Estilos añadidos por Gamus ---
  Jpop,
  Vocaloid,

  /// Variante libre para estilos no incluidos en la lista.
  Custom(String),
}

impl FromStr for Style {
  type Err = std::convert::Infallible;

  /// Intenta convertir una cadena a [`Style`], asignando variantes conocidas
  /// o creando una [`Style::Custom`] si el valor no coincide con ninguna.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let normalized = s.trim().to_lowercase().replace(['-', ' '], "");

    let style = match normalized.as_str() {
      "poprock" => Style::PopRock,
      "house" => Style::House,
      "vocal" => Style::Vocal,
      "experimental" => Style::Experimental,
      "punk" => Style::Punk,
      "alternativerock" => Style::AlternativeRock,
      "synthpop" => Style::SynthPop,
      "techno" => Style::Techno,
      "indierock" => Style::IndieRock,
      "ambient" => Style::Ambient,
      "soul" => Style::Soul,
      "disco" => Style::Disco,
      "hardcore" => Style::Hardcore,
      "folk" => Style::Folk,
      "ballad" => Style::Ballad,
      "country" => Style::Country,
      "hardrock" => Style::HardRock,
      "electro" => Style::Electro,
      "rock&roll" | "rockandroll" => Style::RockAndRoll,
      "chanson" => Style::Chanson,
      "romantic" => Style::Romantic,
      "trance" => Style::Trance,
      "heavymetal" => Style::HeavyMetal,
      "psychedelicrock" => Style::PsychedelicRock,
      "folkrock" => Style::FolkRock,

      // Estilos personalizados
      "jpop" => Style::Jpop,
      "vocaloid" => Style::Vocaloid,

      // Caso general
      _ => Style::Custom(s.to_string()),
    };

    Ok(style)
  }
}

impl fmt::Display for Style {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Style::PopRock => write!(f, "Pop Rock"),
      Style::House => write!(f, "House"),
      Style::Vocal => write!(f, "Vocal"),
      Style::Experimental => write!(f, "Experimental"),
      Style::Punk => write!(f, "Punk"),
      Style::AlternativeRock => write!(f, "Alternative Rock"),
      Style::SynthPop => write!(f, "Synth-pop"),
      Style::Techno => write!(f, "Techno"),
      Style::IndieRock => write!(f, "Indie Rock"),
      Style::Ambient => write!(f, "Ambient"),
      Style::Soul => write!(f, "Soul"),
      Style::Disco => write!(f, "Disco"),
      Style::Hardcore => write!(f, "Hardcore"),
      Style::Folk => write!(f, "Folk"),
      Style::Ballad => write!(f, "Ballad"),
      Style::Country => write!(f, "Country"),
      Style::HardRock => write!(f, "Hard Rock"),
      Style::Electro => write!(f, "Electro"),
      Style::RockAndRoll => write!(f, "Rock & Roll"),
      Style::Chanson => write!(f, "Chanson"),
      Style::Romantic => write!(f, "Romantic"),
      Style::Trance => write!(f, "Trance"),
      Style::HeavyMetal => write!(f, "Heavy Metal"),
      Style::PsychedelicRock => write!(f, "Psychedelic Rock"),
      Style::FolkRock => write!(f, "Folk Rock"),
      Style::Jpop => write!(f, "J-pop"),
      Style::Vocaloid => write!(f, "Vocaloid"),
      Style::Custom(s) => write!(f, "{}", s),
    }
  }
}
