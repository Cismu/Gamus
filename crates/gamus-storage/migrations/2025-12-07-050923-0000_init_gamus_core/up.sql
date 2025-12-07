-- ARTISTS
CREATE TABLE artists (
  id TEXT NOT NULL PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  bio TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE artist_variations (
  id TEXT NOT NULL PRIMARY KEY,
  artist_id TEXT NOT NULL,
  variation TEXT NOT NULL,
  FOREIGN KEY (artist_id) REFERENCES artists(id)
);

CREATE UNIQUE INDEX ux_artist_variations_artist_variation
  ON artist_variations (artist_id, variation);

CREATE TABLE artist_sites (
  id TEXT NOT NULL PRIMARY KEY,
  artist_id TEXT NOT NULL,
  url TEXT NOT NULL,
  FOREIGN KEY (artist_id) REFERENCES artists(id)
);

CREATE UNIQUE INDEX ux_artist_sites_artist_url
  ON artist_sites (artist_id, url);

-- SONGS
CREATE TABLE songs (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  acoustid TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- RELEASES
CREATE TABLE releases (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  release_date TEXT,
  country TEXT,
  notes TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE release_main_artists (
  id TEXT NOT NULL PRIMARY KEY,
  release_id TEXT NOT NULL,
  artist_id TEXT NOT NULL,
  FOREIGN KEY (release_id) REFERENCES releases(id),
  FOREIGN KEY (artist_id) REFERENCES artists(id)
);

CREATE UNIQUE INDEX ux_release_main_artists_release_artist
  ON release_main_artists (release_id, artist_id);

-- RELEASE TYPES
CREATE TABLE release_types (
  id TEXT NOT NULL PRIMARY KEY,
  release_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  FOREIGN KEY (release_id) REFERENCES releases(id)
);

CREATE UNIQUE INDEX ux_release_types_release_kind
  ON release_types (release_id, kind);

-- GENRES & STYLES
CREATE TABLE release_genres (
  id TEXT NOT NULL PRIMARY KEY,
  release_id TEXT NOT NULL,
  genre TEXT NOT NULL,
  FOREIGN KEY (release_id) REFERENCES releases(id)
);

CREATE UNIQUE INDEX ux_release_genres_release_genre
  ON release_genres (release_id, genre);

CREATE TABLE release_styles (
  id TEXT NOT NULL PRIMARY KEY,
  release_id TEXT NOT NULL,
  style TEXT NOT NULL,
  FOREIGN KEY (release_id) REFERENCES releases(id)
);

CREATE UNIQUE INDEX ux_release_styles_release_style
  ON release_styles (release_id, style);

-- ARTWORKS
CREATE TABLE artworks (
  id TEXT NOT NULL PRIMARY KEY,
  release_id TEXT NOT NULL,
  path TEXT NOT NULL,
  mime_type TEXT NOT NULL,
  description TEXT,
  hash TEXT,
  credits TEXT,
  FOREIGN KEY (release_id) REFERENCES releases(id)
);

CREATE UNIQUE INDEX ux_artworks_release_path
  ON artworks (release_id, path);

-- RELEASE TRACKS
CREATE TABLE release_tracks (
  id TEXT NOT NULL PRIMARY KEY,
  release_id TEXT NOT NULL,
  song_id TEXT NOT NULL,
  disc_number INTEGER NOT NULL DEFAULT 1,
  track_number INTEGER NOT NULL,
  title_override TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (release_id) REFERENCES releases(id),
  FOREIGN KEY (song_id) REFERENCES songs(id)
);

CREATE UNIQUE INDEX ux_release_tracks_release_disc_track
  ON release_tracks (release_id, disc_number, track_number);

-- ARTIST ROLES POR PISTA
-- ArtistRole lo modelamos como TEXT en SQLite
CREATE TABLE release_track_artists (
  id TEXT NOT NULL PRIMARY KEY,
  release_track_id TEXT NOT NULL,
  artist_id TEXT NOT NULL,
  role TEXT NOT NULL,
  position INTEGER,
  FOREIGN KEY (release_track_id) REFERENCES release_tracks(id),
  FOREIGN KEY (artist_id) REFERENCES artists(id)
);

CREATE UNIQUE INDEX ux_release_track_artists_track_artist_role
  ON release_track_artists (release_track_id, artist_id, role);

-- ARCHIVOS FÍSICOS & DETALLES TÉCNICOS
CREATE TABLE library_files (
  id TEXT NOT NULL PRIMARY KEY,
  release_track_id TEXT NOT NULL,
  path TEXT NOT NULL UNIQUE,
  size_bytes INTEGER NOT NULL,
  modified_unix INTEGER NOT NULL,
  duration_ms INTEGER NOT NULL,
  bitrate_kbps INTEGER,
  sample_rate_hz INTEGER,
  channels INTEGER,
  fingerprint TEXT,
  bpm REAL,
  quality_score REAL,
  quality_assessment TEXT,
  features BLOB,
  added_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (release_track_id) REFERENCES release_tracks(id)
);
