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
