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

  pub fn import_full(&self) -> Result<(), CoreError> {
    // 1) scanner.scan_library_files()
    // 2) para cada ScannedFile:
    //      - metadata.extract_from_path()
    //      - guardar Song, Release, ReleaseTrack, LibraryFile vía repo
    // 3) manejar errores parciales, logs, etc.
    Ok(())
  }
}
