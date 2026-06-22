---
status: idea
type: feature
priority: low
area: backend
---

Problem: `gallery-backend` treats the filesystem and redb as two systems that
should stay in sync, but nothing watches for drift in case of crashes or FS
damage.

Consider adding additinal robustness checks or perhaps an idle
background job matching FS status against DB records.

See also FS/DB single transaction idea.
