---
status: open
type: chore
priority: high
area: testing
---

Verify backend handling of basic file options via API and via filesystem

- Filesystem image created / removed / moved
  - paths: watcher, manual indexing
  - conflicts: api image move into manually created file (not indexed)
  - use cases to verify (check design.md)
    - new file, index and done
    - new file with same hash, add to alias[]
    - removed file, need to look for aliases to handle potential alias[0] change

- API image upload, move, delete
  - upload to target folder (rename/merge/skip)
  - move to target folder (rename/merge/skip)
  - delete (mark trash, confirm to delete)

