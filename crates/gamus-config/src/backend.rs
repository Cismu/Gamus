use crate::paths::{ConfigError, GamusPaths};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;

/// NUEVO: usa toml_edit para escritura preservando comentarios
use toml_edit::{DocumentMut, Item};

pub trait ConfigBackend {
  fn load_section<T: DeserializeOwned>(&self, section: &str) -> Result<T, ConfigError>;
  fn save_section<T: Serialize>(&self, section: &str, value: &T) -> Result<(), ConfigError>;
}

pub struct TomlConfigBackend {
  paths: GamusPaths,
}

impl TomlConfigBackend {
  pub fn new(paths: GamusPaths) -> Self {
    Self { paths }
  }

  pub fn load_section_with_default<T>(&self, section: &str) -> Result<T, ConfigError>
  where
    T: DeserializeOwned + Default,
  {
    use std::io::ErrorKind;

    let path = self.paths.config_file();
    let content = match std::fs::read_to_string(&path) {
      Ok(c) => c,
      Err(e) if e.kind() == ErrorKind::NotFound => {
        return Ok(T::default());
      }
      Err(e) => return Err(e.into()),
    };

    // Aquí puedes seguir con `toml` normal sin problema
    let toml_val: toml::Value = toml::from_str(&content)?;

    let Some(table) = toml_val.get(section) else {
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

    // 1) Leer config actual como DocumentMut o crear doc vacío si no existe.
    let mut doc: DocumentMut = match fs::read_to_string(&path) {
      Ok(content) => content
        .parse::<DocumentMut>()
        .map_err(|e| ConfigError::Other(format!("parse toml_edit doc: {e}")))?,
      Err(e) if e.kind() == ErrorKind::NotFound => {
        // documento nuevo
        DocumentMut::new()
      }
      Err(e) => return Err(e.into()),
    };

    // 2) Serializar el valor de la sección con `toml` normal (serde) a string.
    let section_str = toml::to_string(value)
      .map_err(|e| ConfigError::Other(format!("encode section [{section}]: {e}")))?;

    // 3) Parsear esa representación parcial a `toml_edit::Item`.
    //    Ojo: `section_str` suele tener formato:
    //      "foo = 1\nbar = 2\n"
    //    que es una tabla "inline" sin cabecera.
    let section_item: Item = section_str
      .parse::<DocumentMut>()
      .map_err(|e| ConfigError::Other(format!("parse section as doc: {e}")))?
      .into_item(); // convierte el DocumentMut a Item (tabla)

    // 4) Insertar / reemplazar la sección en la raíz preservando comentarios externos.
    doc[section] = section_item;

    // 5) Serializar el documento completo preservando comentarios/espacios.
    let serialized = doc.to_string();

    // 6) Escritura atómica usando gamus-fs.
    gamus_fs::atomic_write_str(&path, &serialized)?;

    Ok(())
  }
}
