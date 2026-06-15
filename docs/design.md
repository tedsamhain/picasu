# Goals and Design Philosophy

## Galleries come and go - photos do not

- option to keep original photos totally untouched (R/O)
- keep added metadata in interoperable formats (adjust folders, add xmp sidecars,...?)
- sufficient to backup the photo repo, no worry about DB consistency/versioning

## Design around specific use cases and support them well

- personal photo gallery and backup
  - upload pre-processed photos directly to target albums (file sharing with global write access)
  - upload to configured ingress folder (automated via app or filesharing, sort/manage via gallery, no global write API)

- shared gallery with trusted family and friends
  - manage users and private/shared album access, including to ingress folder

- potentially extend to social streaming/sharing or even federated sharing?

## Keep it robust, low footprint, modular

- best practice devops: memory safe languages, static checks, code smell, depency audits
- backend fully self-contained, API supports scripting and alternate frontends
- frontend should focus on major supported use cases, clean and simple

- further processing/features can be done as sidecars working on backend API
  - analyze photos, add tags, make stories

## Albums

### How albums work

Each image or video belongs to exactly one album, which corresponds to the directory it lives in on disk. The album hierarchy is the filesystem directory tree under each configured `sync_path`. Albums are not stored as separate records in the database — they are derived at runtime from the filesystem via `DIR_ALBUM_CACHE` (a path → albumId map, rebuilt on startup).

Consequence: moving a file to a different album moves it on disk. Refreshing the frontend after a filesystem move reflects the change without re-indexing.

### How to create an album

Albums are created by creating subdirectories. From the UI: open "Move to Album" on any item, select a parent album, enter a name, and press the folder-plus button. This creates a physical subdirectory on disk and registers it as a new album.

Top-level albums correspond to `sync_path` entries in the server config and cannot be created from the UI — they are added by configuring a new sync root and restarting the backend.

### Album invariants

- One file → one album → one directory. Multi-album membership does not exist.
- Moving a file to a different album via the API/UI moves the physical file on disk.
- Top-level albums = sync root directories; sub-albums = subdirectories.
- `DIR_ALBUM_CACHE` is ephemeral. It is rebuilt from the filesystem on every startup and is not persisted in the database.
- Album membership is singular and authoritative: the database record stores `album: Option<ArrayString<64>>`, not a set.

## Further ideas

- photo app should work as local gallery with cache
- integrate/interoperate port knocking
- stories - generate virtual albums (tags?) based on similar location/date
- streams - support social streaming and sharing endpoints (activitypub? chatbots?)

- storage and transfer optimizations
  - push compressed images on mobile, replace with high-res original later    
  - report storage per album, keep track of raw vs post-processed/compressed
  - offer some reasonable default compression ratios
  - detect duplicate / redundant, offer to select best
