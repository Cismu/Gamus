use crate::schema::artists;
use crate::schema::songs;

use diesel::prelude::*;

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

pub struct ReleaseRow {
  pub id: String,
  pub title: String,
  pub release_date: Option<String>,
  pub country: Option<String>,
  pub notes: Option<String>,
  pub created_at: String,
  pub updated_at: String,
}
