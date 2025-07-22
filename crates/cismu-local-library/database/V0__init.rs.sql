BEGIN;

CREATE TABLE IF NOT EXISTS artists (
  id          INTEGER PRIMARY KEY,
  name        TEXT NOT NULL UNIQUE,
  bio         TEXT,
  variations  TEXT,
  sites       TEXT
);

CREATE TABLE IF NOT EXISTS songs (
  id          INTEGER PRIMARY KEY,
  acoustid    TEXT UNIQUE,
  title       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS releases (
  id          INTEGER PRIMARY KEY,
  title       TEXT NOT NULL,
  format      TEXT NOT NULL,
  release_date TEXT
);

CREATE TABLE IF NOT EXISTS release_tracks (
  id                  INTEGER PRIMARY KEY,
  song_id             INTEGER NOT NULL,
  release_id          INTEGER NOT NULL,
  track_number        INTEGER NOT NULL,
  disc_number         INTEGER NOT NULL,
  path                TEXT NOT NULL UNIQUE,
  title_override      TEXT,
  size_bytes          INTEGER NOT NULL,
  modified_timestamp  INTEGER NOT NULL,
  duration_seconds    REAL NOT NULL,
  bitrate_kbps        INTEGER,
  sample_rate_hz      INTEGER,
  channels            INTEGER,
  fingerprint         TEXT,
  bpm                 REAL,
  quality_score       REAL,
  quality_assessment  TEXT,
  features            BLOB,
  FOREIGN KEY(song_id)   REFERENCES songs(id)    ON DELETE CASCADE,
  FOREIGN KEY(release_id) REFERENCES releases(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS song_credits (
  song_id    INTEGER NOT NULL,
  artist_id  INTEGER NOT NULL,
  role       TEXT NOT NULL,
  PRIMARY KEY(song_id, artist_id, role),
  FOREIGN KEY(song_id)   REFERENCES songs(id)    ON DELETE CASCADE,
  FOREIGN KEY(artist_id) REFERENCES artists(id)  ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS release_main_artists (
  release_id INTEGER NOT NULL,
  artist_id  INTEGER NOT NULL,
  PRIMARY KEY(release_id, artist_id),
  FOREIGN KEY(release_id) REFERENCES releases(id) ON DELETE CASCADE,
  FOREIGN KEY(artist_id)  REFERENCES artists(id)  ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS release_genres (
  release_id INTEGER NOT NULL,
  genre      TEXT NOT NULL,
  PRIMARY KEY(release_id, genre),
  FOREIGN KEY(release_id) REFERENCES releases(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS release_styles (
  release_id INTEGER NOT NULL,
  style      TEXT NOT NULL,
  PRIMARY KEY(release_id, style),
  FOREIGN KEY(release_id) REFERENCES releases(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS artworks (
  id          INTEGER PRIMARY KEY,
  release_id  INTEGER NOT NULL,
  path        TEXT NOT NULL,
  mime_type   TEXT,
  description TEXT,
  hash        TEXT UNIQUE,
  credits     TEXT,
  FOREIGN KEY(release_id) REFERENCES releases(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_songs_acoustid ON songs(acoustid);
CREATE INDEX IF NOT EXISTS ix_rt_song        ON release_tracks(song_id);
CREATE INDEX IF NOT EXISTS ix_rt_release     ON release_tracks(release_id);

COMMIT;
