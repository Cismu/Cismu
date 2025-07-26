CREATE TABLE fingerprint_queue (
    release_track_id INTEGER PRIMARY KEY NOT NULL,
    FOREIGN KEY(release_track_id) REFERENCES release_tracks(id) ON DELETE CASCADE
);
