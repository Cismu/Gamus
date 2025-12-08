use crate::domain::artist::Artist;
use crate::domain::release::Release;
use crate::domain::song::Song;
use crate::domain::{ArtistId, ReleaseId, SongId};
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

  // -------- QUERY (read) --------

  pub fn list_artists(&self) -> Result<Vec<Artist>, CoreError> {
    self.repo.list_artists()
  }

  pub fn list_songs(&self) -> Result<Vec<Song>, CoreError> {
    self.repo.list_songs()
  }

  pub fn list_releases(&self) -> Result<Vec<Release>, CoreError> {
    self.repo.list_releases()
  }

  pub fn get_artist(&self, id: ArtistId) -> Result<Option<Artist>, CoreError> {
    self.repo.find_artist(id)
  }

  pub fn get_song(&self, id: SongId) -> Result<Option<Song>, CoreError> {
    self.repo.find_song(id)
  }

  pub fn get_release(&self, id: ReleaseId) -> Result<Option<Release>, CoreError> {
    self.repo.find_release(id)
  }
}
