pub mod models;
pub mod schema;

use std::cell::RefCell;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use gamus_core::domain::artist::Artist;
use gamus_core::domain::ids::ArtistId;
use gamus_core::errors::CoreError;
use gamus_core::ports::LibraryRepository;

use crate::models::{ArtistRow, NewArtistRow};

pub struct SqliteLibraryRepository {
  conn: RefCell<SqliteConnection>,
}

impl SqliteLibraryRepository {
  pub fn new(database_url: &str) -> Result<Self, CoreError> {
    let conn = SqliteConnection::establish(database_url)
      .map_err(|e| CoreError::Repository(e.to_string()))?;
    Ok(Self { conn: RefCell::new(conn) })
  }
}

fn artist_to_new_row(artist: &Artist) -> NewArtistRow {
  NewArtistRow { id: artist.id.to_string(), name: artist.name.clone(), bio: artist.bio.clone() }
}

fn row_to_artist(row: ArtistRow) -> Artist {
  use uuid::Uuid;

  Artist {
    id: ArtistId::from_uuid(Uuid::parse_str(&row.id).expect("invalid uuid in DB")),
    name: row.name,
    variations: vec![],
    bio: row.bio,
    sites: vec![],
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

  fn save_song(&self, _song: &gamus_core::domain::song::Song) -> Result<(), CoreError> {
    unimplemented!()
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

  fn find_song(
    &self,
    _id: gamus_core::domain::ids::SongId,
  ) -> Result<Option<gamus_core::domain::song::Song>, CoreError> {
    unimplemented!()
  }

  fn find_release(
    &self,
    _id: gamus_core::domain::ids::ReleaseId,
  ) -> Result<Option<gamus_core::domain::release::Release>, CoreError> {
    unimplemented!()
  }
}
