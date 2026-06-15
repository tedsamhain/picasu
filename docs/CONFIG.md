# Configuration Guide

Urocissa uses `config.json` for application configuration. This file is located in the `gallery-backend` directory.

If `config.json` does not exist, it will be automatically created with default values when the application starts for the first time.

## Configuration Structure

The configuration is divided into `public` and `private` sections.

```json
{
  "public": {
    "address": "0.0.0.0",
    "port": 5673,
    "limits": {
      "file": "10GiB",
      "json": "10MiB",
      "data-form": "10GiB"
    },
    "syncPaths": [],
    "readOnlyMode": false,
    "disableImg": false
  },
  "private": {
    "password": null,
    "authKey": null,
    "discordHookUrl": null
  }
}
```

## Public Settings

These settings control the server's public-facing behavior.

| Setting            | Type           | Default      | Description                                                                                                                                                          |
| ------------------ | -------------- | ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `address`          | string         | `"0.0.0.0"` | The IP address the server binds to. `"0.0.0.0"` means it listens on all available network interfaces.                                                               |
| `port`             | number         | `5673`       | The port number the server listens on.                                                                                                                               |
| `limits.file`      | string         | `"10GiB"`    | Maximum size for a single file upload. Accepts human-readable sizes: `"1GiB"`, `"512MiB"`, etc.                                                                     |
| `limits.json`      | string         | `"10MiB"`    | Maximum size for JSON request bodies.                                                                                                                                |
| `limits.data-form` | string         | `"10GiB"`    | Maximum size for multipart form submissions (used for photo/video import).                                                                                           |
| `syncPaths`        | array          | `[]`         | Local directory paths to watch for new or changed media files. Example: `["/mnt/photos", "C:\\Users\\Photos"]`                                                      |
| `readOnlyMode`     | boolean        | `false`      | If `true`, the gallery runs in read-only mode — uploads, edits, and deletions are disabled.                                                                          |
| `disableImg`       | boolean        | `false`      | If `true`, disables image processing in the frontend. **Intended for debugging only; do not use in production.**                                                     |

## Private Settings

These settings handle sensitive security and authentication data.

| Setting          | Type           | Default | Description                                                                                                                                                                                                                                                                                                                                     |
| ---------------- | -------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `password`       | string \| null | `null`  | The password required to log in to the web interface. If `null`, no password is set and you will need to configure one.                                                                                                                                                                                                                         |
| `authKey`        | string \| null | `null`  | The secret key used for signing authentication tokens (JWT). <br> - If `null`, a random key is generated on every startup, which invalidates existing login sessions upon restart.<br> - Set this to a random string to persist sessions across server restarts.<br> **If you are unsure what this does, keeping it as `null` is recommended.** |
| `discordHookUrl` | string \| null | `null`  | Optional Discord Webhook URL for receiving error notifications.                                                                                                                                                                                                                                                                                 |

## Advanced: Rocket Web Server Configuration

`config.json` covers the settings most users need to change. Internally, these are merged into the configuration of [Rocket](https://rocket.rs), the web framework Urocissa uses.

Rocket has additional options that are **not** exposed in `config.json` — such as TLS, worker thread count, connection keep-alive, and reverse-proxy header trust. You can set these without modifying the application using either of the two standard Rocket mechanisms, both of which work alongside `config.json`.

### Option 1: `Rocket.toml`

Place a `Rocket.toml` file in the `gallery-backend` directory. Settings here are merged with (and can override) the values from `config.json`.

```toml
[default]
workers = 8          # async worker threads (default: logical CPUs × 2)
keep_alive = 30      # connection keep-alive in seconds
```

> **HTTPS:** The standard build does not include Rocket's built-in TLS. The recommended approach for most deployments is to terminate TLS at a reverse proxy (nginx, Caddy) in front of Urocissa. Caddy handles certificate provisioning and renewal automatically.

### Option 2: Environment variables

Any Rocket option can also be set via a `ROCKET_` prefixed environment variable, which is convenient for containers and systemd units:

```sh
ROCKET_WORKERS=8
ROCKET_KEEP_ALIVE=30
ROCKET_ADDRESS=127.0.0.1  # overrides config.json
ROCKET_PORT=5673           # overrides config.json
```

> **Note:** `ROCKET_ADDRESS` and `ROCKET_PORT` will override the values in `config.json`. Set them in one place only to avoid confusion.

For the full list of available Rocket options see the [Rocket configuration guide](https://rocket.rs/guide/configuration/).
