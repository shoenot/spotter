# spotter 
Spotify listening history logger

Consists of two parts:
    - spotter: authenticates with Spotify, makes requests to the Spotify API for information, stores them in the Postgres database. It polls the spotify API every 5 minutes to get a list of all the songs that have been played since then.
    - spotter-api: reads from the Postgres database (using readymade queries) and makes them available via API endpoints for integration into other stuff (website, etc.)

## Installation  

This is very much a personal project (for now at least) but I dont expect breaking changes, and its easy to set up:
    - ```git clone``` the repo and ```cd``` into it.
    - ```docker compose build```
    - ```docker compose up``` (do not pass the -d flag as you need to authenticate)
    - Go to the authentication link printed by the spotter. Once you authenticate successfully, it will automatically return.
    - You should be ready to go. Press d to detach from the docker logs and enjoy!

## API Reference  

Base URL: `http://<host>:4005`

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/top/artists` | Top artists by play count |
| GET | `/top/albums` | Top albums by play count |
| GET | `/top/tracks` | Top tracks by play count |
| GET | `/history` | Full play history for a date range |
| GET | `/recents` | 25 most recently played tracks |
| GET | `/stats` | Aggregate listening statistics |

---

### Common Notes

- All timestamps are ISO 8601 with timezone offset, e.g. `2025-01-01T00:00:00+06:00`
- Endpoints that accept `from` / `to` default to the start of the current month and the current moment (Dhaka time, UTC+6) when omitted
- All endpoints return JSON arrays — an empty result returns `[]` (HTTP 200), not 404
- Server errors return HTTP 500 with no body
- `play_time` values are in **minutes**

---

### GET `/top/artists`

Returns the top artists ranked by play count within a time window.

**Query Parameters**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `from` | datetime | start of current month | Start of the time range (inclusive) |
| `to` | datetime | now (Dhaka) | End of the time range (exclusive) |
| `lim` | integer | `10` | Maximum number of artists to return |

**Response Fields**

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Artist name |
| `play_count` | integer | Number of plays within the time range |
| `play_time` | integer | Total listening time in minutes within the time range |
| `first_listened` | datetime | Timestamp of the artist's first ever play (all-time history, not within the range provided) |
| `first_listened_track` | string | Name of the track played at that first listen |

> `first_listened` and `first_listened_track` reflect the all-time earliest play, regardless of the `from`/`to` window.

---

### GET `/top/albums`

Returns the top albums ranked by play count within a time window.

**Query Parameters**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `from` | datetime | start of current month | Start of the time range (inclusive) |
| `to` | datetime | now (Dhaka) | End of the time range (exclusive) |
| `lim` | integer | `10` | Maximum number of albums to return |

**Response Fields**

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Album name |
| `play_count` | integer | Number of plays within the time range |
| `play_time` | integer | Total listening time in minutes within the time range |
| `artists` | string | Comma-separated list of the album's credited artists, sorted alphabetically |
| `first_listened` | datetime | Timestamp of the album's first ever play (all-time) |
| `first_listened_track` | string | Name of the track played at that first listen |

---

### GET `/top/tracks`

Returns the top tracks ranked by play count within a time window.

**Query Parameters**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `from` | datetime | start of current month | Start of the time range (inclusive) |
| `to` | datetime | now (Dhaka) | End of the time range (exclusive) |
| `lim` | integer | `10` | Maximum number of tracks to return |

**Response Fields**

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Track name |
| `play_count` | integer | Number of plays within the time range |
| `play_time` | integer | Total listening time in minutes within the time range |
| `artists` | string | Comma-separated list of the track's credited artists, sorted alphabetically |
| `album` | string | Album name (first alphabetically if the track belongs to multiple) |
| `first_listened` | datetime | Timestamp of the track's first ever play (all-time) |

---

### GET `/history`

Returns the full play history for a given time range, ordered most recent first.

> Unlike the `/top/*` endpoints, `from` and `to` are **required** — there are no defaults.

**Query Parameters**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `from` | datetime | **required** | Start of the time range (inclusive) |
| `to` | datetime | **required** | End of the time range (exclusive) |

**Response Fields**

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Track name |
| `artists` | string | Comma-separated list of artists, sorted alphabetically |
| `album` | string | Album name (first alphabetically if multiple) |
| `played_at` | datetime | Timestamp when the track was played |

---

### GET `/recents`

Returns the 25 most recently played tracks across all time. Takes no parameters.

**Response Fields**

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Track name |
| `artists` | string | Comma-separated list of artists, sorted alphabetically |
| `album` | string | Album name (first alphabetically if multiple) |
| `played_at` | datetime | Timestamp when the track was played |

---

### GET `/stats`

Returns aggregate listening statistics for a time range. Always returns a single-element array.

**Query Parameters**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `from` | datetime | start of current month | Start of the time range (inclusive) |
| `to` | datetime | now (Dhaka) | End of the time range (exclusive) |

**Response Fields**

| Field | Type | Description |
|-------|------|-------------|
| `total_plays` | integer | Total number of plays in the range |
| `unique_tracks` | integer | Number of distinct tracks played |
| `unique_artists` | integer | Number of distinct artists played |
| `unique_albums` | integer | Number of distinct albums played |
| `total_playtime` | string | Formatted total listening time, e.g. `14h 32m` |
| `avg_release_year` | string | Average release year of played tracks, rounded to nearest year — `"0"` if unavailable |

> The time range for `/stats` is filtered by date boundary rather than exact timestamps, so plays on the boundary dates are always fully included.
