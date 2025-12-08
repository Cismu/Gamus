pub mod config;
pub mod models;
pub mod schema;

use std::path::PathBuf;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{MigrationHarness, embed_migrations};
use uuid::Uuid;

use gamus_core::domain::{
  ArtistId, ReleaseId, SongId, artist::Artist, release::Release, song::Song,
};
use gamus_core::errors::CoreError;
use gamus_core::ports::LibraryRepository;

// Importamos los modelos y el schema
use crate::models::{ArtistRow, NewArtistRow, NewReleaseRow, NewSongRow, ReleaseRow, SongRow};

pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations = embed_migrations!("migrations");

// Definimos un alias para el tipo de Pool para no escribirlo todo el tiempo
type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

#[derive(Clone)]
pub struct SqliteLibraryRepository {
  pool: SqlitePool,
}

impl SqliteLibraryRepository {
  pub fn new(db_path: &PathBuf, journal_mode: &Option<String>) -> Result<Self, CoreError> {
    let db_path = db_path.to_str().ok_or(CoreError::Repository("Invalid db path".to_string()))?;
    let manager = ConnectionManager::<SqliteConnection>::new(db_path);

    // Configuramos el pool
    let pool = r2d2::Pool::builder()
      // Configuración recomendada para SQLite en apps de escritorio:
      // Testea la conexión al iniciar para asegurar que el archivo existe/es accesible
      .test_on_check_out(true)
      .build(manager)
      .map_err(|e| CoreError::Repository(format!("Pool error: {}", e)))?;

    // Ejecutamos migraciones y configuraciones iniciales
    // Obtenemos una conexión del pool temporalmente
    let mut conn = pool.get().map_err(|e| CoreError::Repository(e.to_string()))?;

    // Activamos WAL mode para mejor concurrencia (Lecturas y escrituras simultáneas)
    if let Some(mode) = journal_mode {
      diesel::sql_query(format!("PRAGMA journal_mode = {}", mode))
        .execute(&mut conn)
        .map_err(|e| CoreError::Repository(format!("wal error: {}", e)))?;
    }

    conn
      .run_pending_migrations(MIGRATIONS)
      .map_err(|e| CoreError::Repository(format!("migration error: {e}")))?;

    Ok(Self { pool })
  }

  pub fn new_from_config() -> Result<Self, CoreError> {
    use crate::config::StorageConfig;

    let cfg = StorageConfig::load().map_err(|e| CoreError::Repository(e.to_string()))?;

    Self::new(&cfg.db_path, &cfg.journal_mode)
  }

  // Helper interno para obtener conexión de forma breve
  fn get_conn(
    &self,
  ) -> Result<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, CoreError> {
    self.pool.get().map_err(|e| CoreError::Repository(format!("connection error: {}", e)))
  }
}

// --- Métodos de consulta directos (fuera del trait si los necesitas públicos) ---
impl SqliteLibraryRepository {
  pub fn list_artists(&self) -> Result<Vec<Artist>, CoreError> {
    use crate::schema::artists::dsl::*;
    let mut conn = self.get_conn()?;

    let rows: Vec<ArtistRow> =
      artists.load::<ArtistRow>(&mut conn).map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(rows.into_iter().map(row_to_artist).collect())
  }

  pub fn list_songs(&self) -> Result<Vec<Song>, CoreError> {
    use crate::schema::songs::dsl::*;
    let mut conn = self.get_conn()?;

    let rows =
      songs.load::<SongRow>(&mut conn).map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(rows.into_iter().map(row_to_song).collect())
  }
}

// --- Implementación del Trait ---
impl LibraryRepository for SqliteLibraryRepository {
  fn save_artist(&self, artist: &Artist) -> Result<(), CoreError> {
    use crate::schema::artists::dsl::*;

    let new_row = artist_to_new_row(artist);
    let mut conn = self.get_conn()?;

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
}

// --- Helpers de Conversión (Mantenidos igual, solo movidos al final para limpieza) ---
// Nota: Es buena práctica que estos implementen el trait `From` o `Into`
// en lugar de ser funciones sueltas, pero así como están funcionan bien.

fn artist_to_new_row(artist: &Artist) -> NewArtistRow {
  NewArtistRow { id: artist.id.to_string(), name: artist.name.clone(), bio: artist.bio.clone() }
}

fn song_to_new_row(song: &Song) -> NewSongRow {
  NewSongRow { id: song.id.to_string(), title: song.title.clone(), acoustid: song.acoustid.clone() }
}

fn release_to_new_row(release: &Release) -> NewReleaseRow {
  NewReleaseRow {
    id: release.id.to_string(),
    title: release.title.clone(),
    release_date: release.release_date.clone(),
  }
}

fn row_to_artist(row: ArtistRow) -> Artist {
  Artist {
    id: ArtistId::from_uuid(Uuid::parse_str(&row.id).expect("Invalid UUID")),
    name: row.name,
    variations: vec![],
    bio: row.bio,
    sites: vec![],
  }
}

fn row_to_song(row: SongRow) -> Song {
  Song {
    id: SongId::from_uuid(Uuid::parse_str(&row.id).expect("Invalid UUID")),
    title: row.title,
    acoustid: row.acoustid,
  }
}

fn row_to_release(row: ReleaseRow) -> Release {
  Release {
    id: ReleaseId::from_uuid(Uuid::parse_str(&row.id).expect("Invalid UUID")),
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
