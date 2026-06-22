---
status: open
type: feature
priority: medium
area: frontend
---

## UI Refinement

Consolidated from `ui-notes.md` — remaining work after nav/settings cleanup.

### Navigation

- **Auto-hide nav behavior**: only collapse on very slim screens; otherwise keep fixed with a toggle button to switch modes.
- **Breadcrumbs**: show current album/image path in the top bar or a nav directory tree.

### Search / Filter bar

- On click, show a dropdown to help build filters (tag picker, album picker, property toggles).
- Unify global photo properties (favorite, trashed) as the same filter type, selectable via the search bar dropdown.

### Album view

- Album properties in grid view: allow setting a fancy name and custom cover image.
- Quick favorite labeling from grid view.

### Config / Settings

- Rework "Image Path" and "Album Index":
  - On first start (empty DB), set up a watcher on `IMAGE_HOME`.
  - Offer to index `IMAGE_HOME` from the main page.
  - Detect whether FS notify is available; fall back to manual indexing on networked filesystems.
- Delete flow: mark as trashed (visible in trash bin), require confirmation on final delete.
- Smart move: discard if identical target file exists, auto-rename otherwise.

### Misc

- Add `$HOME` shortcut to the "Add Folder" dialog.
- Verify image/album move operation works correctly.
