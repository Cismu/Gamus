----------------------------------------------------------
-- 1. IDENTITY & ARTISTS
----------------------------------------------------------
CREATE TABLE artists (
  id TEXT PRIMARY KEY NOT NULL, -- UUID v4
  name TEXT NOT NULL,
  bio TEXT,
  -- Fechas como TEXT para mapear directo a String en Rust
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE artist_variations (
  id TEXT PRIMARY KEY NOT NULL,
  artist_id TEXT NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
  variation TEXT NOT NULL,
  UNIQUE(artist_id, variation)
);

CREATE TABLE artist_sites (
  id TEXT PRIMARY KEY NOT NULL,
  artist_id TEXT NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
  url TEXT NOT NULL,
  UNIQUE(artist_id, url)
);

----------------------------------------------------------
-- 2. SONGS & STATS
----------------------------------------------------------
CREATE TABLE songs (
  id TEXT PRIMARY KEY NOT NULL,
  title TEXT NOT NULL,
  acoustid TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE song_comments (
  id TEXT PRIMARY KEY NOT NULL,
  song_id TEXT NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
  comment TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE song_ratings (
  id TEXT PRIMARY KEY NOT NULL,
  song_id TEXT NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
  value_fixed_point INTEGER NOT NULL, -- u32 Fixed Point
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

----------------------------------------------------------
-- 3. RELEASES (PRODUCT)
----------------------------------------------------------
CREATE TABLE releases (
  id TEXT PRIMARY KEY NOT NULL,
  title TEXT NOT NULL,
  release_date TEXT, -- Ya era TEXT, se mantiene
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE release_types (
  id TEXT PRIMARY KEY NOT NULL,
  release_id TEXT NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  UNIQUE(release_id, kind)
);

CREATE TABLE release_main_artists (
  id TEXT PRIMARY KEY NOT NULL,
  release_id TEXT NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
  artist_id TEXT NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
  UNIQUE(release_id, artist_id)
);

CREATE TABLE release_genres (
  id TEXT PRIMARY KEY NOT NULL,
  release_id TEXT NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
  genre TEXT NOT NULL,
  UNIQUE(release_id, genre)
);

CREATE TABLE release_styles (
  id TEXT PRIMARY KEY NOT NULL,
  release_id TEXT NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
  style TEXT NOT NULL,
  UNIQUE(release_id, style)
);

CREATE TABLE artworks (
  id TEXT PRIMARY KEY NOT NULL,
  release_id TEXT NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
  path TEXT NOT NULL,
  mime_type TEXT NOT NULL,
  description TEXT,
  hash TEXT,
  credits TEXT,
  UNIQUE(release_id, path)
);

----------------------------------------------------------
-- 4. TRACKS (PHYSICAL INSTANCE)
----------------------------------------------------------
CREATE TABLE release_tracks (
  id TEXT PRIMARY KEY NOT NULL,
  release_id TEXT NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
  song_id TEXT NOT NULL REFERENCES songs(id) ON DELETE RESTRICT,
  disc_number INTEGER NOT NULL DEFAULT 1,
  track_number INTEGER NOT NULL,
  title_override TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(release_id, disc_number, track_number)
);

----------------------------------------------------------
-- 5. CREDITS & ROLES
----------------------------------------------------------
CREATE TABLE release_track_artists (
  id TEXT PRIMARY KEY NOT NULL,
  release_track_id TEXT NOT NULL REFERENCES release_tracks(id) ON DELETE CASCADE,
  artist_id TEXT NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
  role TEXT NOT NULL,
  position INTEGER,
  UNIQUE(release_track_id, artist_id, role)
);

----------------------------------------------------------
-- 6. FILES & TECHNICAL DETAILS
----------------------------------------------------------
CREATE TABLE library_files (
  id TEXT PRIMARY KEY NOT NULL,
  release_track_id TEXT NOT NULL REFERENCES release_tracks(id) ON DELETE CASCADE,
  
  -- File Details
  path TEXT NOT NULL UNIQUE,
  size_bytes BIGINT NOT NULL,
  modified_unix BIGINT NOT NULL,
  
  -- Audio Details
  duration_ms BIGINT NOT NULL,
  bitrate_kbps INTEGER,
  sample_rate_hz INTEGER,
  channels INTEGER,
  fingerprint TEXT,
  
  -- Audio Analysis / Quality
  bpm REAL,
  quality_score REAL,
  quality_assessment TEXT,
  features BLOB, 
  
  added_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(release_track_id)
);