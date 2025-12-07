use crate::errors::CoreError;
use crate::ports::{FileScanner, LibraryRepository, MetadataExtractor};

pub struct LibraryService<S, M, R>
where
  S: FileScanner,
  M: MetadataExtractor,
  R: LibraryRepository,
{
  scanner: S,
  metadata: M,
  repo: R,
}

impl<S, M, R> LibraryService<S, M, R>
where
  S: FileScanner,
  M: MetadataExtractor,
  R: LibraryRepository,
{
  pub fn new(scanner: S, metadata: M, repo: R) -> Self {
    Self { scanner, metadata, repo }
  }

  /// Importa la biblioteca completa:
  /// - escanea archivos
  /// - extrae metadatos
  /// - persiste Song / Release / ReleaseTrack (lo que tengas implementado)
  pub async fn import_full(&self) -> Result<(), CoreError> {
    // 1) Escanear archivos (async)
    let groups =
      self.scanner.scan_library_files().await.map_err(|e| CoreError::Scan(e.to_string()))?;

    for group in groups {
      for scanned in group.files {
        // 2) Extraer metadatos para cada archivo (async)
        let extracted = self
          .metadata
          .extract_from_path(&scanned.path)
          .await
          .map_err(|e| CoreError::Metadata(e.to_string()))?;

        // 3) Guardar Song
        self.repo.save_song(&extracted.song).map_err(|e| CoreError::Repository(e.to_string()))?;

        // 4) Guardar Release (si existe)
        if let Some(release) = &extracted.release {
          self.repo.save_release(release).map_err(|e| CoreError::Repository(e.to_string()))?;
        }

        // 5) Guardar ReleaseTrack / LibraryFile
        //    (todavía no tienes esos métodos en el repo, pero la idea sería algo así):
        //
        // if let Some(track) = &extracted.track {
        //   self
        //     .repo
        //     .save_release_track(track)
        //     .map_err(|e| CoreError::Repository(e.to_string()))?;
        //
        //   let file_details = FileDetails {
        //     path: scanned.path.clone(),
        //     size: scanned.size_bytes,
        //     modified: scanned.modified_unix,
        //   };
        //
        //   self
        //     .repo
        //     .save_library_file(track.id, &file_details)
        //     .map_err(|e| CoreError::Repository(e.to_string()))?;
        // }
      }
    }

    Ok(())
  }
}
