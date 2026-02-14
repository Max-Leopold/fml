# Factorio Mod Portal API

The Mod Portal API allows programmatic browsing and downloading of mods from the official Factorio mod portal.

**No authentication is required** to browse or search mods. Authentication (username + token) is only needed for downloading mod files and managing bookmarks.

All responses are `Content-Type: application/json`. Responses are cached for up to 15 minutes (`Cache-Control: max-age=900, public`).

---

## Base URL

```
https://mods.factorio.com
```

All endpoints below are relative to this base URL.

---

## Endpoints

### List Mods

```
GET /api/mods
```

Returns a paginated list of mods with optional filtering and sorting.

#### Query Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `hide_deprecated` | boolean | — | If set, only return non-deprecated mods. |
| `page` | integer | `1` | Page number (1-indexed). Out-of-range pages return 200 with empty `results`. |
| `page_size` | integer \| `"max"` | `25` | Results per page. `"max"` returns all results in one response and sets `pagination` to `null`. |
| `sort` | string | `"name"` | Sort field. One of: `name`, `created_at`, `updated_at`. |
| `sort_order` | string | `"desc"` | Sort direction. One of: `asc`, `desc`. |
| `namelist` | string[] | — | Filter to specific mod names. Comma-separated or repeat the parameter. **Changes response shape:** returns `releases` array instead of `latest_release`, omits `thumbnail`. |
| `version` | string | — | Filter to mods compatible with a Factorio version. One of: `0.13`, `0.14`, `0.15`, `0.16`, `0.17`, `0.18`, `1.0`, `1.1`, `2.0`. |

#### Response

Returns a [Mod List Response](#mod-list-response).

---

### Get Mod (Short)

```
GET /api/mods/{mod_name}
```

Returns basic information about a specific mod. Includes all releases but no dependency info, changelog, or timestamps.

#### Response

Returns a [Result Entry](#result-entry) (short-level fields). Returns HTTP 404 on unknown mod.

---

### Get Mod (Full)

```
GET /api/mods/{mod_name}/full
```

Returns comprehensive information about a mod including description, changelog, tags, license, images, FAQ, and release details with dependency information.

#### Response

Returns a [Result Entry](#result-entry) (all fields). Returns HTTP 404 on unknown mod.

---

### List Bookmarks

```
GET /api/bookmarks?username={username}&token={token}
```

**Requires authentication.** Returns a JSON array of bookmarked mod name strings.

---

### Toggle Bookmark

```
GET /api/bookmarks/toggle?username={username}&token={token}&mod={mod_name}&state={state}
```

**Requires authentication.** Sets the bookmark state for a mod.

| Parameter | Type | Description |
|---|---|---|
| `username` | string | Factorio account username. |
| `token` | string | Authentication token. |
| `mod` | string | Machine-readable mod name. |
| `state` | string | `on` or `off`. |

---

## Response Objects

### Mod List Response

Top-level response from `GET /api/mods`.

| Field | Type | Description |
|---|---|---|
| `pagination` | [Pagination](#pagination) \| null | Pagination metadata. `null` when `page_size=max`. |
| `results` | [Result Entry](#result-entry)[] | Array of mod entries. Empty array if page is out of range. |

---

### Pagination

| Field | Type | Description |
|---|---|---|
| `count` | integer | Total number of mods matching the query. |
| `links` | [Pagination Links](#pagination-links) | Navigation URLs. |
| `page` | integer | Current page number. |
| `page_count` | integer | Total number of pages. |
| `page_size` | integer | Results per page. |

---

### Pagination Links

All values are absolute URLs or `null` when not applicable (e.g. `prev` is `null` on the first page).

| Field | Type | Description |
|---|---|---|
| `first` | string \| null | URL of the first page. |
| `prev` | string \| null | URL of the previous page. |
| `next` | string \| null | URL of the next page. |
| `last` | string \| null | URL of the last page. |

---

### Result Entry

The available fields depend on which endpoint returned the entry. Fields marked with `?` after the type are omitted from the response when their value is null/empty/false — do not assume they exist.

- **List** = `GET /api/mods` (without `namelist`)
- **Namelist** = `GET /api/mods` (with `namelist`)
- **Short** = `GET /api/mods/{name}`
- **Full** = `GET /api/mods/{name}/full`

| Field | Type | List | Namelist | Short | Full | Description |
|---|---|---|---|---|---|---|
| `name` | string | yes | yes | yes | yes | Unique machine-readable mod identifier. |
| `title` | string | yes | yes | yes | yes | Human-readable display name. |
| `owner` | string | yes | yes | yes | yes | Username of the mod author. |
| `summary` | string | yes | yes | yes | yes | Short description (max 500 chars). |
| `downloads_count` | integer | yes | yes | yes | yes | Total number of downloads. |
| `category` | string | yes | yes | yes | yes | Category. See [Categories](#categories). |
| `score` | float | yes | yes | yes | yes | Mod ranking score. Can be negative or zero. |
| `latest_release` | [Release](#release) | yes | — | — | — | Most recent release. Only on list endpoint without `namelist`. |
| `releases` | [Release](#release)[] | — | yes | yes | yes | All releases. On list endpoint, only present with `namelist`. See note on [info_json](#infojson) differences. |
| `thumbnail` | string? | yes | — | yes | yes | Relative thumbnail path. Construct full URL: `https://assets-mod.factorio.com{thumbnail}`. |
| `last_highlighted_at` | string? | — | yes | yes | yes | Date when mod was last featured. Format: `YYYY-MM-DD` (date only, not a full timestamp). |
| `changelog` | string? | — | — | — | yes | Full changelog text. |
| `created_at` | string | — | — | — | yes | ISO 8601 timestamp, e.g. `2020-05-24T19:15:48.523000Z`. |
| `updated_at` | string | — | — | — | yes | ISO 8601 timestamp. |
| `description` | string? | — | — | — | yes | Full description text (may contain markdown). |
| `faq` | string? | — | — | — | yes | FAQ text (may contain markdown, or a URL). |
| `images` | [Image](#image)[] | — | — | — | yes | Gallery images. Empty array if none. |
| `tags` | string[] | — | — | — | yes | Tag strings. See [Tags](#tags). Empty array if none. |
| `license` | [License](#license) | — | — | — | yes | License information (single object, not an array). |
| `homepage` | string? | — | — | — | yes | Project homepage URL. |
| `source_url` | string? | — | — | — | yes | Source code repository URL. |
| `github_path` | string? | — | — | — | yes | **Deprecated.** GitHub `owner/repo` path. Use `source_url` instead. |
| `deprecated` | boolean? | — | — | — | yes | Only present (as `true`) when the mod is deprecated. Absent when not deprecated. |

---

### Release

| Field | Type | Description |
|---|---|---|
| `download_url` | string | Relative download path starting with `/download`. See [Downloading Mods](#downloading-mods). |
| `file_name` | string | Filename, typically `{mod_name}_{version}.zip`. |
| `info_json` | [InfoJson](#infojson) | Parsed `info.json` from the mod zip. Content varies by endpoint. |
| `released_at` | string | ISO 8601 timestamp, e.g. `2020-05-24T19:15:48.520000Z`. |
| `version` | string | Mod version string, e.g. `"1.2.3"`. |
| `sha1` | string | SHA-1 hex digest of the zip file. |
| `feature_flags` | string[]? | Feature flags required by the mod, e.g. `["space-travel"]`. Only present when non-empty. |

---

### InfoJson

Content varies by endpoint:

- **List / Namelist / Short endpoints**: Only contains `factorio_version`.
- **Full endpoint**: Also includes `dependencies`.

| Field | Type | Endpoints | Description |
|---|---|---|---|
| `factorio_version` | string | All | Target Factorio version, e.g. `"2.0"`, `"1.1"`. |
| `dependencies` | string[] | Full only | Dependency strings. See [Dependency Format](#dependency-format). |

---

### Image

| Field | Type | Description |
|---|---|---|
| `id` | string | Image asset hash identifier. |
| `thumbnail` | string | Full thumbnail URL, e.g. `https://assets-mod.factorio.com/assets/{id}.thumb.png`. |
| `url` | string | Full image URL, e.g. `https://assets-mod.factorio.com/assets/{id}.png`. |

---

### License

A single object (not an array).

| Field | Type | Description |
|---|---|---|
| `id` | string | License identifier, e.g. `"default_mit"`, `"custom_5a9d..."`. |
| `name` | string | Short name, e.g. `"mit"`. |
| `title` | string | Display title, e.g. `"MIT"`. |
| `description` | string | Full license text or summary. |
| `url` | string | URL to the license text. |

Standard license IDs: `default_mit`, `default_gnugplv3`, `default_gnulgplv3`, `default_mozilla2`, `default_apache2`, `default_unlicense`. Custom licenses use the format `custom_{id}`.

---

### Categories

| Value | Description |
|---|---|
| `no-category` | No category / uncategorized. |
| `content` | New game content (items, entities, etc.). |
| `overhaul` | Large overhaul / total conversion mods. |
| `tweaks` | Balance, gameplay, or graphics tweaks. |
| `utilities` | Tools, interface adjustments, QoL. |
| `scenarios` | Scenarios, maps, puzzles. |
| `mod-packs` | Collections of mods. |
| `localizations` | Translation / localization mods. |
| `internal` | Lua libraries for other mods. |

---

### Tags

Tags are plain strings in the `tags` array. Known values:

`transportation`, `logistics`, `trains`, `combat`, `armor`, `enemies`, `environment`, `mining`, `fluids`, `logistic-network`, `circuit-network`, `manufacturing`, `power`, `storage`, `blueprints`, `cheats`

---

### Dependency Format

Dependency strings in `info_json.dependencies` follow this syntax:

```
[prefix] mod-name [operator version]
```

**Prefixes:**

| Prefix | Meaning |
|---|---|
| *(none)* | Hard requirement. Mod will not load without it. |
| `!` | Incompatible. Cannot be loaded together (version is ignored). |
| `?` | Optional. Loaded after if present, no error if missing. |
| `(?)` | Hidden optional. Same as `?` but not shown to players. |
| `~` | Does not affect load order. |

**Operators:** `<`, `<=`, `=`, `>=`, `>`

**Examples:**
```
"base >= 2.0.0"        -- hard dependency on base version 2.0.0+
"? quality"            -- optional dependency, any version
"! incompatible-mod"   -- incompatible with this mod
"(?) hidden-lib"       -- hidden optional dependency
"~ some-mod >= 1.0.0"  -- no load order dependency
"space-age"            -- hard dependency, any version
```

---

## Error Responses

### 404 Not Found

Returned when requesting a non-existent mod via `/api/mods/{name}` or `/api/mods/{name}/full`:

```json
{
  "message": "Mod not found"
}
```

### Out-of-range Page

Requesting a page beyond `page_count` returns HTTP 200 with an empty results array:

```json
{
  "pagination": {
    "count": 19925,
    "page": 99999,
    "page_count": 9963,
    "page_size": 25,
    "links": {
      "first": "https://mods.factorio.com/api/mods?page_size=25",
      "next": null,
      "prev": "https://mods.factorio.com/api/mods?page=99998&page_size=25",
      "last": null
    }
  },
  "results": []
}
```

---

## Downloading Mods

Downloading mod files **requires authentication**. Without credentials, the server redirects to a login page.

### Download URL

```
GET https://mods.factorio.com{download_url}?username={username}&token={token}
```

`download_url` comes from the [Release](#release) object, e.g. `/download/flib/5ecac7e44d121d000cd77c76`.

---

## Authentication

### Obtaining a Token

**Option 1: player-data.json**

The Factorio client stores credentials in `player-data.json` in the [User Data directory](https://wiki.factorio.com/Application_directory#User_data_directory). Extract the `service-token` field.

**Option 2: Web Authentication API**

```
POST https://auth.factorio.com/api-login
Content-Type: application/x-www-form-urlencoded
```

| Parameter | Required | Description |
|---|---|---|
| `username` | yes | Account username or email. |
| `password` | yes | Account password. |
| `api_version` | no | API version (default `1`, latest `6`). |
| `require_game_ownership` | no | Set `true` to verify Factorio is purchased. |
| `email_authentication_code` | no | Required if previous attempt returned `email-authentication-required`. |

**Success response** (API version >= 2):
```json
{
  "token": "abc123hextoken",
  "username": "player_name"
}
```

**Error response:**
```json
{
  "error": "login-failed",
  "message": "Invalid username or password.",
  "data": {}
}
```

Known error codes: `login-failed`, `email-authentication-required`.

HTTP status codes: API versions <= 3 return 401 on error; versions >= 4 return 200 for both success and error. Excessive failed attempts are rate-limited.

---

## JSON Response Examples

### GET /api/mods?page_size=2&sort=updated_at

```json
{
  "pagination": {
    "count": 19925,
    "page": 1,
    "page_count": 9963,
    "page_size": 2,
    "links": {
      "first": null,
      "next": "https://mods.factorio.com/api/mods?page_size=2&page=2",
      "prev": null,
      "last": "https://mods.factorio.com/api/mods?page_size=2&page=9963"
    }
  },
  "results": [
    {
      "name": "FastRunning",
      "title": "RunningFaster",
      "owner": "easygoing",
      "summary": "increase running speed",
      "downloads_count": 11,
      "category": "tweaks",
      "score": 0,
      "latest_release": {
        "download_url": "/download/FastRunning/5a5f1ae6adcc441024d73231",
        "file_name": "FastRunning_1.0.0.zip",
        "info_json": {
          "factorio_version": "0.13"
        },
        "released_at": "2016-06-28T11:42:48.071000Z",
        "version": "1.0.0",
        "sha1": "28a87dc62028f56da0717e99f29432c929ef3387"
      }
    }
  ]
}
```

### GET /api/mods?namelist=flib

```json
{
  "pagination": null,
  "results": [
    {
      "name": "flib",
      "title": "Factorio Library",
      "owner": "raiguard",
      "summary": "A set of high-quality, commonly-used utilities for creating Factorio mods.",
      "downloads_count": 984870,
      "category": "internal",
      "score": -141.9,
      "releases": [
        {
          "download_url": "/download/flib/5ecac7e44d121d000cd77c76",
          "file_name": "flib_0.1.0.zip",
          "info_json": {
            "factorio_version": "0.18"
          },
          "released_at": "2020-05-24T19:15:48.520000Z",
          "sha1": "55f7bbcfc0c0e831008b57c321db509bf3a25285",
          "version": "0.1.0"
        }
      ]
    }
  ]
}
```

### GET /api/mods/flib (short)

```json
{
  "name": "flib",
  "title": "Factorio Library",
  "owner": "raiguard",
  "summary": "A set of high-quality, commonly-used utilities for creating Factorio mods.",
  "downloads_count": 984870,
  "category": "internal",
  "score": -141.9,
  "thumbnail": "/assets/0a42d642ea743a66354d74b7f1d2bc1d70e9449e.thumb.png",
  "releases": [
    {
      "download_url": "/download/flib/5ecac7e44d121d000cd77c76",
      "file_name": "flib_0.1.0.zip",
      "info_json": {
        "factorio_version": "0.18"
      },
      "released_at": "2020-05-24T19:15:48.520000Z",
      "sha1": "55f7bbcfc0c0e831008b57c321db509bf3a25285",
      "version": "0.1.0"
    }
  ]
}
```

### GET /api/mods/flib/full (truncated)

```json
{
  "name": "flib",
  "title": "Factorio Library",
  "owner": "raiguard",
  "summary": "A set of high-quality, commonly-used utilities for creating Factorio mods.",
  "downloads_count": 984870,
  "category": "internal",
  "score": -141.9,
  "thumbnail": "/assets/0a42d642ea743a66354d74b7f1d2bc1d70e9449e.thumb.png",
  "created_at": "2020-05-24T19:15:48.523000Z",
  "updated_at": "2025-11-05T17:00:00.000000Z",
  "changelog": "---------------------------------------------------------------------------------------------------\nVersion: 0.16.5\nDate: 2025-11-05\n  Bugfixes:\n    - Fixed that...\n",
  "description": "# Factorio Library\n\nThe Factorio Library is a set of high-quality...",
  "faq": "**What is the difference between this and the Factorio Standard Library?**\n...",
  "homepage": "https://codeberg.org/raiguard/flib",
  "source_url": "https://codeberg.org/raiguard/flib",
  "github_path": "factoriolib/flib",
  "images": [],
  "tags": [],
  "license": {
    "id": "default_mit",
    "name": "mit",
    "title": "MIT",
    "description": "A permissive license...",
    "url": "https://opensource.org/licenses/MIT"
  },
  "releases": [
    {
      "download_url": "/download/flib/5ecac7e44d121d000cd77c76",
      "file_name": "flib_0.1.0.zip",
      "info_json": {
        "factorio_version": "0.18",
        "dependencies": [
          "base >= 0.18.19"
        ]
      },
      "released_at": "2020-05-24T19:15:48.520000Z",
      "sha1": "55f7bbcfc0c0e831008b57c321db509bf3a25285",
      "version": "0.1.0"
    }
  ]
}
```

### Release with feature_flags (from full endpoint)

```json
{
  "download_url": "/download/global-power-network/...",
  "file_name": "global-power-network_0.0.1.zip",
  "info_json": {
    "factorio_version": "2.0",
    "dependencies": ["base", "space-age", "? quality"]
  },
  "released_at": "2025-01-15T12:00:00.000000Z",
  "sha1": "...",
  "version": "0.0.1",
  "feature_flags": ["space-travel"]
}
```

### Image object (from full endpoint)

```json
{
  "id": "3788dc49d95c030ca36a6b0d9269abe5a91e62af",
  "thumbnail": "https://assets-mod.factorio.com/assets/3788dc49d95c030ca36a6b0d9269abe5a91e62af.thumb.png",
  "url": "https://assets-mod.factorio.com/assets/3788dc49d95c030ca36a6b0d9269abe5a91e62af.png"
}
```

### 404 error

```json
{
  "message": "Mod not found"
}
```
