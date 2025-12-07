use crate::paths::{ConfigError, GamusPaths};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;

pub trait ConfigBackend {
  /// Carga una sección (tabla) del TOML como un tipo arbitrario.
  fn load_section<T: DeserializeOwned>(&self, section: &str) -> Result<T, ConfigError>;

  /// Opcional: guardar cambios (no lo necesitas hoy, pero deja el contrato listo).
  fn save_section<T: Serialize>(&self, section: &str, value: &T) -> Result<(), ConfigError>;
}

/// Implementación que usa un gamus.toml con varias tablas:
///
/// ```toml
/// [fs]
/// audio_exts = ["mp3", "flac"]
///
/// [storage]
/// db_filename = "gamus.db"
/// ```
pub struct TomlConfigBackend {
  paths: GamusPaths,
}

impl TomlConfigBackend {
  pub fn new(paths: GamusPaths) -> Self {
    Self { paths }
  }

  /// Versión "gentil" que:
  /// - si no existe el archivo gamus.toml → devuelve `T::default()`
  /// - si no existe la sección → devuelve `T::default()`
  /// - si hay un error de parseo → sigue devolviendo error (para no tapar bugs feos)
  pub fn load_section_with_default<T>(&self, section: &str) -> Result<T, ConfigError>
  where
    T: DeserializeOwned + Default,
  {
    use std::io::ErrorKind;

    let path = self.paths.config_file();
    let content = match std::fs::read_to_string(&path) {
      Ok(c) => c,
      Err(e) if e.kind() == ErrorKind::NotFound => {
        // no hay gamus.toml → usar defaults
        return Ok(T::default());
      }
      Err(e) => return Err(e.into()),
    };

    let toml_val: toml::Value = toml::from_str(&content)?;

    let Some(table) = toml_val.get(section) else {
      // falta [section] → usar defaults
      return Ok(T::default());
    };

    let t: T = table
      .clone()
      .try_into()
      .map_err(|e| ConfigError::Other(format!("decode section [{section}]: {e}")))?;

    Ok(t)
  }
}

impl ConfigBackend for TomlConfigBackend {
  fn load_section<T: DeserializeOwned>(&self, section: &str) -> Result<T, ConfigError> {
    let path = self.paths.config_file();
    let content = fs::read_to_string(&path)?;
    let toml_val: toml::Value = toml::from_str(&content)?;

    let table = toml_val
      .get(section)
      .ok_or_else(|| ConfigError::Other(format!("missing section [{section}] in {:?}", path)))?;

    let t: T = table
      .clone()
      .try_into()
      .map_err(|e| ConfigError::Other(format!("decode section [{section}]: {e}")))?;

    Ok(t)
  }

  fn save_section<T: Serialize>(&self, _section: &str, _value: &T) -> Result<(), ConfigError> {
    // lo puedes implementar luego, por ahora no lo necesitas
    Err(ConfigError::Other("save_section not implemented".into()))
  }
}
