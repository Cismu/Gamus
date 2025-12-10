pub mod config;
pub mod models;
pub mod schema;

use std::path::PathBuf;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{MigrationHarness, embed_migrations};
use uuid::Uuid;

use gamus_core::domain::{ArtistId, ReleaseId, SongId, artist::Artist, release::Release, song::Song};
use gamus_core::errors::CoreError;
use gamus_core::ports::Library;

use crate::models::{ArtistRow, NewArtistRow, NewReleaseRow, NewSongRow, ReleaseRow, SongRow};

/// Embeds migration SQL files into the compiled binary for self-contained execution.
pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations = embed_migrations!("migrations");

type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

/// Concrete implementation of the `Library` port backed by SQLite.
///
/// Uses `r2d2` for connection pooling to manage file handles efficiently in a desktop environment.
/// This struct is cheap to clone as it wraps an `Arc` to the pool.
#[derive(Clone)]
pub struct LibraryStore {
  pool: SqlitePool,
}

impl LibraryStore {
  /// Initializes the store, sets up the connection pool, runs pending migrations,
  /// and applies SQLite optimization pragmas.
  ///
  /// # Arguments
  ///
  /// * `db_path` - Filesystem path to the SQLite database.
  /// * `journal_mode` - Optional PRAGMA journal_mode setting (defaults to WAL if passed).
  ///
  /// # Security & Concurrency
  ///
  /// * Enables `test_on_check_out` to handle filesystem volatility common in desktop apps (e.g., file locks, deletion).
  /// * Applies WAL mode to allow non-blocking concurrent reads while writing.
  pub fn new(db_path: &PathBuf, journal_mode: &Option<String>) -> Result<Self, CoreError> {
    // Validate path encoding early to prevent runtime IO errors downstream
    let db_path = db_path.to_str().ok_or(CoreError::Repository("Invalid db path".to_string()))?;
    let manager = ConnectionManager::<SqliteConnection>::new(db_path);

    let pool = r2d2::Pool::builder()
      // Crucial for desktop context: verifies the connection is still alive and the file
      // is accessible before handing it to a thread. Slightly expensive but prevents "Database Locked" panics.
      .test_on_check_out(true)
      .build(manager)
      .map_err(|e| CoreError::Repository(format!("Pool error: {}", e)))?;

    // Acquire an ephemeral connection for setup tasks
    let mut conn = pool.get().map_err(|e| CoreError::Repository(e.to_string()))?;

    // WAL (Write-Ahead Logging) is critical for concurrency in SQLite.
    // Without this, a write operation locks the entire database file against readers.
    if let Some(mode) = journal_mode {
      diesel::sql_query(format!("PRAGMA journal_mode = {}", mode))
        .execute(&mut conn)
        .map_err(|e| CoreError::Repository(format!("wal error: {}", e)))?;
    }

    conn.run_pending_migrations(MIGRATIONS).map_err(|e| CoreError::Repository(format!("migration error: {e}")))?;

    Ok(Self { pool })
  }

  /// Convenience constructor loading configuration from the environment/file.
  pub fn new_from_config() -> Result<Self, CoreError> {
    use crate::config::StorageConfig;

    let cfg = StorageConfig::load().map_err(|e| CoreError::Repository(e.to_string()))?;

    Self::new(&cfg.db_path, &cfg.journal_mode)
  }

  /// Internal helper to retrieve a connection from the pool.
  ///
  /// # Errors
  /// Returns `CoreError::Repository` if the pool is exhausted or the timeout is reached.
  fn get_conn(&self) -> Result<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, CoreError> {
    self.pool.get().map_err(|e| CoreError::Repository(format!("connection error: {}", e)))
  }
}

impl Library for LibraryStore {
  fn save_artist(&self, artist: &Artist) -> Result<(), CoreError> {
    use crate::schema::artists::dsl::*;

    let new_row = artist_to_new_row(artist);
    let mut conn = self.get_conn()?;

    // UPSERT semantics: Ensure idempotency by updating fields on conflict.
    diesel::insert_into(artists)
      .values(&new_row)
      .on_conflict(id)
      .do_update()
      .set((name.eq(&artist.name), bio.eq(artist.bio.as_deref())))
      .execute(&mut conn)
      .map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(())
  }

  fn save_song(&self, song: &Song) -> Result<(), CoreError> {
    use crate::schema::songs::dsl::*;

    let new_row = song_to_new_row(song);
    let mut conn = self.get_conn()?;

    diesel::insert_into(songs)
      .values(&new_row)
      .on_conflict(id)
      .do_update()
      .set((title.eq(&song.title), acoustid.eq(song.acoustid.as_deref())))
      .execute(&mut conn)
      .map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(())
  }

  fn save_release(&self, release: &Release) -> Result<(), CoreError> {
    use crate::schema::releases::dsl::*;

    let new_row = release_to_new_row(release);
    let mut conn = self.get_conn()?;

    diesel::insert_into(releases)
      .values(&new_row)
      .on_conflict(id)
      .do_update()
      .set((title.eq(&release.title), release_date.eq(release.release_date.as_deref())))
      .execute(&mut conn)
      .map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(())
  }

  fn find_artist(&self, artist_id: ArtistId) -> Result<Option<Artist>, CoreError> {
    use crate::schema::artists::dsl::*;
    use diesel::OptionalExtension;

    let id_str = artist_id.to_string();
    let mut conn = self.get_conn()?;

    let row_opt = artists
      .filter(id.eq(id_str))
      .first::<ArtistRow>(&mut conn)
      .optional()
      .map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(row_opt.map(row_to_artist))
  }

  fn find_song(&self, song_id: SongId) -> Result<Option<Song>, CoreError> {
    use crate::schema::songs::dsl::*;
    use diesel::OptionalExtension;

    let id_str = song_id.to_string();
    let mut conn = self.get_conn()?;

    let row_opt = songs
      .filter(id.eq(id_str))
      .first::<SongRow>(&mut conn)
      .optional()
      .map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(row_opt.map(row_to_song))
  }

  fn find_release(&self, release_id: ReleaseId) -> Result<Option<Release>, CoreError> {
    use crate::schema::releases::dsl::*;
    use diesel::OptionalExtension;

    let id_str = release_id.to_string();
    let mut conn = self.get_conn()?;

    let row_opt = releases
      .filter(id.eq(id_str))
      .first::<ReleaseRow>(&mut conn)
      .optional()
      .map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(row_opt.map(row_to_release))
  }

  fn list_artists(&self) -> Result<Vec<Artist>, CoreError> {
    use crate::schema::artists::dsl::*;
    let mut conn = self.get_conn()?;

    // Note: Loading all rows without pagination may impact memory/performance on large libraries.
    // Consider adding limits/offsets to the `Library` trait interface in the future.
    let rows: Vec<ArtistRow> =
      artists.load::<ArtistRow>(&mut conn).map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(rows.into_iter().map(row_to_artist).collect())
  }

  fn list_songs(&self) -> Result<Vec<Song>, CoreError> {
    use crate::schema::songs::dsl::*;
    let mut conn = self.get_conn()?;

    let rows = songs.load::<SongRow>(&mut conn).map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(rows.into_iter().map(row_to_song).collect())
  }

  fn list_releases(&self) -> Result<Vec<Release>, CoreError> {
    use crate::schema::releases::dsl::*;
    let mut conn = self.get_conn()?;

    let rows = releases.load::<ReleaseRow>(&mut conn).map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(rows.into_iter().map(row_to_release).collect())
  }
}

// --- DTO Mapping Helpers ---
// Decouples Domain Entities (business logic) from Diesel Models (DB schema).

fn artist_to_new_row(artist: &Artist) -> NewArtistRow {
  NewArtistRow { id: artist.id.to_string(), name: artist.name.clone(), bio: artist.bio.clone() }
}

fn song_to_new_row(song: &Song) -> NewSongRow {
  NewSongRow { id: song.id.to_string(), title: song.title.clone(), acoustid: song.acoustid.clone() }
}

fn release_to_new_row(release: &Release) -> NewReleaseRow {
  NewReleaseRow { id: release.id.to_string(), title: release.title.clone(), release_date: release.release_date.clone() }
}

// Inversion mappings (DB -> Domain)
// Assumes DB integrity regarding UUID formatting.
// NOTE: `expect` usage here relies on the invariant that IDs stored are valid UUIDs.
// Database corruption could cause panics here.

fn row_to_artist(row: ArtistRow) -> Artist {
  Artist {
    id: ArtistId::from_uuid(Uuid::parse_str(&row.id).expect("Invalid UUID in database")),
    name: row.name,
    variations: vec![],
    bio: row.bio,
    sites: vec![],
  }
}

fn row_to_song(row: SongRow) -> Song {
  Song {
    id: SongId::from_uuid(Uuid::parse_str(&row.id).expect("Invalid UUID in database")),
    title: row.title,
    acoustid: row.acoustid,
  }
}

fn row_to_release(row: ReleaseRow) -> Release {
  Release {
    id: ReleaseId::from_uuid(Uuid::parse_str(&row.id).expect("Invalid UUID in database")),
    title: row.title,
    release_type: vec![],
    main_artist_ids: vec![],
    release_tracks: vec![],
    release_date: row.release_date,
    artworks: vec![],
    genres: vec![],
    styles: vec![],
  }
}
