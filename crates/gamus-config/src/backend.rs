use crate::paths::{ConfigError, GamusPaths};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::io::Write;

pub trait ConfigBackend {
  /// Carga una sección (tabla) del TOML como un tipo arbitrario.
  fn load_section<T: DeserializeOwned>(&self, section: &str) -> Result<T, ConfigError>;

  /// Guarda (crea o reemplaza) una sección completa.
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

  fn save_section<T: Serialize>(&self, section: &str, value: &T) -> Result<(), ConfigError> {
    use std::io::ErrorKind;

    let path = self.paths.config_file();

    // 1) Leer config actual, o crear documento vacío si no existe.
    let mut root: toml::Value = match fs::read_to_string(&path) {
      Ok(content) => toml::from_str(&content)?,
      Err(e) if e.kind() == ErrorKind::NotFound => {
        // archivo no existe → empezamos con tabla raíz vacía
        toml::Value::Table(toml::map::Map::new())
      }
      Err(e) => return Err(e.into()),
    };

    // 2) Asegurarnos de que la raíz es una tabla.
    let root_table = root.as_table_mut().ok_or_else(|| {
      ConfigError::Other(format!("config file {:?} does not contain a TOML table at root", path))
    })?;

    // 3) Serializar la sección a toml::Value.
    let section_val = toml::Value::try_from(value)
      .map_err(|e| ConfigError::Other(format!("encode section [{section}]: {e}")))?;

    // 4) Insertar o reemplazar la tabla de esa sección.
    root_table.insert(section.to_string(), section_val);

    // 5) Serializar todo el documento de vuelta a String.
    let serialized = toml::to_string_pretty(&root)
      .map_err(|e| ConfigError::Other(format!("serialize toml: {e}")))?;

    // 6) Escritura atómica: escribir a archivo temporal y renombrar.
    let tmp_path = path.with_extension("tmp");

    {
      let mut tmp_file = fs::File::create(&tmp_path)?;
      tmp_file.write_all(serialized.as_bytes())?;
      tmp_file.sync_all()?;
    }

    fs::rename(&tmp_path, &path)?;

    Ok(())
  }
}
