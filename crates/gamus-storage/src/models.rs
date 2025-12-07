use crate::schema::artists;
use crate::schema::releases;
use crate::schema::songs;

use diesel::prelude::*;

// ====================
// ARTISTS
// ====================

#[derive(Debug, Queryable)]
#[diesel(table_name = artists)]
pub struct ArtistRow {
  pub id: String,
  pub name: String,
  pub bio: Option<String>,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = artists)]
pub struct NewArtistRow {
  pub id: String,
  pub name: String,
  pub bio: Option<String>,
}

// ====================
// SONGS
// ====================

#[derive(Debug, Queryable)]
#[diesel(table_name = songs)]
pub struct SongRow {
  pub id: String,
  pub title: String,
  pub acoustid: Option<String>,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = songs)]
pub struct NewSongRow {
  pub id: String,
  pub title: String,
  pub acoustid: Option<String>,
}

// ====================
// RELEASES
// ====================

#[derive(Debug, Queryable)]
#[diesel(table_name = releases)]
pub struct ReleaseRow {
  pub id: String,
  pub title: String,
  pub release_date: Option<String>,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = releases)]
pub struct NewReleaseRow {
  pub id: String,
  pub title: String,
  pub release_date: Option<String>,
}
