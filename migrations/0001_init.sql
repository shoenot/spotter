CREATE TABLE artists (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    image_url   TEXT,
    artist_url  TEXT
);

CREATE TABLE albums (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    album_type      TEXT,
    release_date    DATE,
    image_url       TEXT,
    album_url       TEXT
);

CREATE TABLE tracks (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    duration        INTEGER,
    popularity      INTEGER,
    track_url       TEXT
);

CREATE TABLE album_artists (
    album_id        TEXT REFERENCES albums(id),
    artist_id       TEXT REFERENCES artists(id),
    PRIMARY KEY (album_id, artist_id)
);

CREATE TABLE track_artists (
    track_id        TEXT REFERENCES tracks(id),
    artist_id       TEXT REFERENCES artists(id),
    PRIMARY KEY (track_id, artist_id)
);

CREATE TABLE track_albums (
    track_id        TEXT REFERENCES tracks(id),
    album_id        TEXT REFERENCES albums(id),
    PRIMARY KEY (track_id, album_id)
);

CREATE TABLE plays (
    id              BIGSERIAL PRIMARY KEY,
    track_id        TEXT NOT NULL REFERENCES tracks(id),
    played_at       TIMESTAMPTZ UNIQUE NOT NULL
);

