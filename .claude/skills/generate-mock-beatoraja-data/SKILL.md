---
name: generate-mock-beatoraja-data
description: Generate mock beatoraja DB files for development and testing. Use this skill when the user wants to create test data, mock beatoraja databases, or set up a local development environment with sample play data.
---

# Generate Mock beatoraja Data

Generate mock beatoraja SQLite DB files and table cache files for development and testing.

## Overview

This skill creates a complete set of beatoraja mock data:

- `songdata.db` -- song metadata (8 songs)
- `player/mock_player/scoredatalog.db` -- latest play records per song
- `player/mock_player/score.db` -- best scores
- `player/mock_player/scorelog.db` -- best-update event log
- `table/` -- difficulty table `.bmt` files (gzip-compressed JSON)

## Constraints

- Output directory: `/tmp/mock-beatoraja/`
- Tools: `sqlite3` CLI and `python3` (for gzip compression)
- Idempotent: delete and recreate on every run
- `date` fields must use the current time so plays register as "today" (based on 05:00 reset time)

## Procedure

### Step 1: Set up directory structure

```bash
MOCK_DIR="/tmp/mock-beatoraja"
rm -rf "$MOCK_DIR"
mkdir -p "$MOCK_DIR/player/mock_player" "$MOCK_DIR/table"
```

### Step 2: Define song data

Use 8 songs with BMS-style titles and artists. Each song needs a unique sha256 hash (use deterministic hex strings for reproducibility).

| #   | sha256 (64-char hex) | title              | artist        | difficulty  | level | notes |
| --- | -------------------- | ------------------ | ------------- | ----------- | ----- | ----- |
| 1   | `a`\*64              | Ascension          | DJ TEKINA     | 3 (HYPER)   | 8     | 800   |
| 2   | `b`\*64              | Brain Detonation   | sironoize     | 4 (ANOTHER) | 11    | 1200  |
| 3   | `c`\*64              | Chrono Nexus       | Frozen Candle | 4 (ANOTHER) | 12    | 1500  |
| 4   | `d`\*64              | DUAL BREAKER       | xi vs FALL    | 5 (INSANE)  | 14    | 1800  |
| 5   | `e`\*64              | Eternal Rave       | SHIKI         | 3 (HYPER)   | 9     | 900   |
| 6   | `f`\*64              | Frozen Dystopia    | Cranky        | 4 (ANOTHER) | 10    | 1100  |
| 7   | `1a1a`\*16           | Gravity Collapse   | LeaF          | 5 (INSANE)  | 15    | 2000  |
| 8   | `2b2b`\*16           | Hyper Luminescence | Kobaryo       | 4 (ANOTHER) | 12    | 1400  |

### Step 3: Create songdata.db

```sql
CREATE TABLE song (
    md5 TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    title TEXT,
    subtitle TEXT,
    genre TEXT,
    artist TEXT,
    subartist TEXT,
    tag TEXT,
    path TEXT PRIMARY KEY,
    folder TEXT,
    stagefile TEXT,
    banner TEXT,
    backbmp TEXT,
    preview TEXT,
    parent TEXT,
    level INTEGER,
    difficulty INTEGER,
    maxbpm INTEGER,
    minbpm INTEGER,
    length INTEGER,
    mode INTEGER,
    judge INTEGER,
    feature INTEGER,
    content INTEGER,
    date INTEGER,
    favorite INTEGER,
    adddate INTEGER,
    notes INTEGER,
    charthash TEXT
);
```

Insert each song with:

- `md5`: use `md5_` + first 28 chars of sha256
- `path`: use `/bms/songs/{title}/{title}_7k.bme`
- `mode`: 0 (7KEYS)
- `genre`: 'BMS'
- `maxbpm`, `minbpm`: 150-200 range
- `length`: 90000-150000 ms
- Other optional fields: NULL

### Step 4: Create scoredatalog.db (today's plays)

```sql
CREATE TABLE scoredatalog (
    sha256 TEXT NOT NULL,
    mode INTEGER NOT NULL,
    clear INTEGER NOT NULL,
    epg INTEGER NOT NULL,
    egr INTEGER NOT NULL,
    egd INTEGER NOT NULL,
    epr INTEGER NOT NULL,
    emr INTEGER NOT NULL,
    ems INTEGER NOT NULL,
    lpg INTEGER NOT NULL,
    lgr INTEGER NOT NULL,
    lgd INTEGER NOT NULL,
    lpr INTEGER NOT NULL,
    lmr INTEGER NOT NULL,
    lms INTEGER NOT NULL,
    minbp INTEGER NOT NULL,
    notes INTEGER NOT NULL,
    combo INTEGER NOT NULL,
    date INTEGER NOT NULL,
    PRIMARY KEY (sha256, mode)
);
```

Assign clear lamps to cover all types. Use the current UNIX timestamp (in seconds) for `date`. The EX score formula is: `epg*2 + egr + lpg*2 + lgr`.

| Song # | clear         | Description  | epg | egr | lpg | lgr | minbp | combo |
| ------ | ------------- | ------------ | --- | --- | --- | --- | ----- | ----- |
| 1      | 1 (Failed)    | Failed       | 200 | 100 | 150 | 80  | 50    | 300   |
| 2      | 4 (Easy)      | Easy clear   | 400 | 150 | 300 | 100 | 30    | 600   |
| 3      | 5 (Normal)    | Normal clear | 500 | 200 | 400 | 150 | 20    | 900   |
| 4      | 6 (Hard)      | Hard clear   | 600 | 250 | 500 | 180 | 15    | 1200  |
| 5      | 7 (ExHard)    | ExHard clear | 350 | 100 | 280 | 70  | 10    | 800   |
| 6      | 8 (FullCombo) | Full Combo   | 450 | 120 | 350 | 80  | 0     | 1100  |
| 7      | 9 (Perfect)   | Perfect      | 800 | 50  | 700 | 30  | 0     | 2000  |
| 8      | 10 (Max)      | Max          | 700 | 0   | 700 | 0   | 0     | 1400  |

Set `egd, epr, emr, ems, lgd, lpr, lmr, lms` to 0 for all rows. Set `notes` to match the song's notes count.

### Step 5: Create score.db (best scores)

```sql
CREATE TABLE score (
    sha256 TEXT NOT NULL,
    mode INTEGER,
    clear INTEGER,
    epg INTEGER,
    lpg INTEGER,
    egr INTEGER,
    lgr INTEGER,
    egd INTEGER,
    lgd INTEGER,
    ebd INTEGER,
    lbd INTEGER,
    epr INTEGER,
    lpr INTEGER,
    ems INTEGER,
    lms INTEGER,
    notes INTEGER,
    combo INTEGER,
    minbp INTEGER,
    avgjudge INTEGER NOT NULL DEFAULT 2147483647,
    playcount INTEGER,
    clearcount INTEGER,
    trophy TEXT,
    ghost TEXT,
    option INTEGER,
    seed INTEGER,
    random INTEGER,
    date INTEGER,
    state INTEGER,
    scorehash TEXT,
    PRIMARY KEY (sha256, mode)
);
```

Insert best scores matching scoredatalog data. Set `playcount` to 5-20 and `clearcount` to 1-10 for each song.

### Step 6: Create scorelog.db (clear lamp update events)

```sql
CREATE TABLE scorelog (
    sha256 TEXT NOT NULL,
    mode INTEGER,
    clear INTEGER,
    oldclear INTEGER,
    score INTEGER,
    oldscore INTEGER,
    combo INTEGER,
    oldcombo INTEGER,
    minbp INTEGER,
    oldminbp INTEGER,
    date INTEGER
);
```

Insert clear lamp updates for a subset of songs to enable split-bar display testing. Include songs where the clear lamp improved today:

| Song # | oldclear      | clear (new) | oldscore | score | oldminbp | minbp |
| ------ | ------------- | ----------- | -------- | ----- | -------- | ----- |
| 3      | 4 (Easy)      | 5 (Normal)  | 2000     | 2250  | 30       | 20    |
| 4      | 5 (Normal)    | 6 (Hard)    | 2500     | 2830  | 25       | 15    |
| 5      | 6 (Hard)      | 7 (ExHard)  | 1800     | 2060  | 15       | 10    |
| 7      | 8 (FullCombo) | 9 (Perfect) | 3200     | 3380  | 0        | 0     |

Use the same `date` as scoredatalog entries.

### Step 7: Create table .bmt files

Create gzip-compressed JSON files in `table/`:

**satellite.bmt** (Satellite table):

```json
{
  "name": "Satellite",
  "tag": "sl",
  "folder": [
    { "name": "sl0", "songs": [{ "sha256": "<song1_sha256>" }] },
    {
      "name": "sl1",
      "songs": [{ "sha256": "<song2_sha256>" }, { "sha256": "<song5_sha256>" }]
    },
    {
      "name": "sl2",
      "songs": [{ "sha256": "<song3_sha256>" }, { "sha256": "<song6_sha256>" }]
    },
    {
      "name": "sl3",
      "songs": [{ "sha256": "<song4_sha256>" }, { "sha256": "<song8_sha256>" }]
    }
  ]
}
```

**insane.bmt** (Insane BMS table):

```json
{
  "name": "Insane BMS",
  "tag": "insane",
  "folder": [
    {
      "name": "★1",
      "songs": [{ "sha256": "<song1_sha256>" }, { "sha256": "<song5_sha256>" }]
    },
    {
      "name": "★2",
      "songs": [{ "sha256": "<song2_sha256>" }, { "sha256": "<song6_sha256>" }]
    },
    {
      "name": "★3",
      "songs": [{ "sha256": "<song3_sha256>" }, { "sha256": "<song8_sha256>" }]
    },
    {
      "name": "★4",
      "songs": [{ "sha256": "<song4_sha256>" }, { "sha256": "<song7_sha256>" }]
    }
  ]
}
```

Use `python3` to create gzip-compressed files:

```bash
python3 -c "
import gzip, json
data = ... # JSON dict
with gzip.open('path.bmt', 'wt') as f:
    json.dump(data, f)
"
```

### Step 8: Verify

Run the following checks:

```bash
sqlite3 "$MOCK_DIR/songdata.db" "SELECT count(*) FROM song;"
# Expected: 8
sqlite3 "$MOCK_DIR/player/mock_player/scoredatalog.db" "SELECT count(*) FROM scoredatalog;"
# Expected: 8
sqlite3 "$MOCK_DIR/player/mock_player/score.db" "SELECT count(*) FROM score;"
# Expected: 8
sqlite3 "$MOCK_DIR/player/mock_player/scorelog.db" "SELECT count(*) FROM scorelog;"
# Expected: 4
python3 -c "import gzip, json; print(json.loads(gzip.open('$MOCK_DIR/table/satellite.bmt').read())['name'])"
# Expected: Satellite
```

### Step 9: Report

Output the generated directory structure and a summary of the data.
