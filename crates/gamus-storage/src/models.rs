use crate::schema::artists;
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
