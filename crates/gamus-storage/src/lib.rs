pub mod models;
pub mod schema;

use std::cell::RefCell;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use uuid::Uuid;

use gamus_core::domain::{ArtistId, SongId};
use gamus_core::domain::{artist::Artist, song::Song};
use gamus_core::errors::CoreError;
use gamus_core::ports::LibraryRepository;

use crate::models::{ArtistRow, NewArtistRow, NewSongRow, SongRow};

pub struct SqliteLibraryRepository {
  conn: RefCell<SqliteConnection>,
}

impl SqliteLibraryRepository {
  pub fn new(database_url: &str) -> Result<Self, CoreError> {
    let conn = SqliteConnection::establish(database_url)
      .map_err(|e| CoreError::Repository(e.to_string()))?;
    Ok(Self { conn: RefCell::new(conn) })
  }

  /// Devuelve todos los artistas de la base de datos.
  pub fn list_artists(&self) -> Result<Vec<Artist>, CoreError> {
    use crate::schema::artists::dsl::*;

    let mut conn = self.conn.borrow_mut();

    let rows: Vec<ArtistRow> =
      artists.load::<ArtistRow>(&mut *conn).map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(rows.into_iter().map(row_to_artist).collect())
  }
}

fn artist_to_new_row(artist: &Artist) -> NewArtistRow {
  NewArtistRow { id: artist.id.to_string(), name: artist.name.clone(), bio: artist.bio.clone() }
}

fn song_to_new_row(song: &Song) -> NewSongRow {
  NewSongRow { id: song.id.to_string(), title: song.title.clone(), acoustid: song.acoustid.clone() }
}

fn row_to_artist(row: ArtistRow) -> Artist {
  Artist {
    id: ArtistId::from_uuid(Uuid::parse_str(&row.id).expect("invalid uuid in DB")),
    name: row.name,
    variations: vec![],
    bio: row.bio,
    sites: vec![],
  }
}

fn row_to_song(row: SongRow) -> Song {
  Song {
    id: SongId::from_uuid(Uuid::parse_str(&row.id).expect("invalid uuid in DB")),
    title: row.title,
    acoustid: row.acoustid,
  }
}

impl LibraryRepository for SqliteLibraryRepository {
  fn save_artist(&self, artist: &Artist) -> Result<(), CoreError> {
    use crate::schema::artists::dsl::*;

    let new_row = artist_to_new_row(artist);
    let mut conn = self.conn.borrow_mut();

    diesel::insert_into(artists)
      .values(&new_row)
      .on_conflict(id)
      .do_update()
      .set((name.eq(&artist.name), bio.eq(artist.bio.as_deref())))
      .execute(&mut *conn)
      .map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(())
  }

  fn save_song(&self, song: &Song) -> Result<(), CoreError> {
    use crate::schema::songs::dsl::*;

    let new_row = song_to_new_row(song);
    let mut conn = self.conn.borrow_mut();

    diesel::insert_into(songs)
      .values(&new_row)
      .on_conflict(id)
      .do_update()
      .set((title.eq(&song.title), acoustid.eq(song.acoustid.as_deref())))
      .execute(&mut *conn)
      .map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(())
  }

  fn save_release(&self, _release: &gamus_core::domain::release::Release) -> Result<(), CoreError> {
    unimplemented!()
  }

  fn find_artist(&self, artist_id: ArtistId) -> Result<Option<Artist>, CoreError> {
    use crate::schema::artists::dsl::*;
    use diesel::OptionalExtension;

    let id_str = artist_id.to_string();
    let mut conn = self.conn.borrow_mut();

    let row_opt = artists
      .filter(id.eq(id_str))
      .first::<ArtistRow>(&mut *conn)
      .optional()
      .map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(row_opt.map(row_to_artist))
  }

  fn find_song(&self, song_id: SongId) -> Result<Option<Song>, CoreError> {
    use crate::schema::songs::dsl::*;
    use diesel::OptionalExtension;

    let id_str = song_id.to_string();
    let mut conn = self.conn.borrow_mut();

    let row_opt = songs
      .filter(id.eq(id_str))
      .first::<SongRow>(&mut *conn)
      .optional()
      .map_err(|e| CoreError::Repository(e.to_string()))?;

    Ok(row_opt.map(row_to_song))
  }

  fn find_release(
    &self,
    _id: gamus_core::domain::ids::ReleaseId,
  ) -> Result<Option<gamus_core::domain::release::Release>, CoreError> {
    unimplemented!()
  }
}
