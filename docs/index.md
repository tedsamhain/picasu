# picasu

A self-hosted photo gallery for millions of images.

- **Backend:** Rust (Rocket, utoipa, redb)
- **Frontend:** Vue 3 + Vuetify + Pinia + Vue Router

## Quick links

- [Architecture Overview](design.md)
- [Configuration Reference](config.md)
- [API Reference](openapi-reference.md)
- [Rust API Docs](../rustdoc/picasu/index.html) (generated from source)
- [TypeScript API Docs](../typedoc/index.html) (generated from source)

## Development

```sh
just build      # build frontend + backend
just test       # run all tests
just check      # lint + format check
just run        # launch against sandbox/
```
