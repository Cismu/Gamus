use crate::ports::{FileScanner, LibraryRepository, MetadataExtractor};

#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
  #[error("repository error: {0}")]
  Repo(String),
  #[error("scanner error: {0}")]
  Scan(String),
  #[error("metadata error: {0}")]
  Metadata(String),
}

pub struct LibraryService<R, S, M>
where
  R: LibraryRepository,
  S: FileScanner,
  M: MetadataExtractor,
{
  repo: R,
  scanner: S,
  metadata: M,
}

impl<R, S, M> LibraryService<R, S, M>
where
  R: LibraryRepository,
  S: FileScanner,
  M: MetadataExtractor,
{
  pub fn new(repo: R, scanner: S, metadata: M) -> Self {
    Self { repo, scanner, metadata }
  }

  /// Escanea el sistema de archivos, extrae metadatos y guarda en la librería.
  pub fn import_all(&self) -> Result<(), LibraryError> {
    let paths = self.scanner.scan_music_dirs().map_err(|e| LibraryError::Scan(e.to_string()))?;

    for path in paths {
      let (song, release, _) = self
        .metadata
        .extract_from_path(&path)
        .map_err(|e| LibraryError::Metadata(e.to_string()))?;

      self.repo.save_song(&song).map_err(|e| LibraryError::Repo(e.to_string()))?;
      self.repo.save_release(&release).map_err(|e| LibraryError::Repo(e.to_string()))?;
    }

    Ok(())
  }
}
