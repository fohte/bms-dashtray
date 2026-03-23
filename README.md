# bms-dashtray

A real-time play history dashboard for [beatoraja](https://github.com/exch-bms2/beatoraja).

bms-dashtray sits alongside beatoraja and watches its database files. Every time you finish a song, the result pops up instantly — clear lamp, EX score diff, BP diff, and whether you set a new personal best. No manual refresh, no browser tab, just a lightweight always-on-top window.

Great for streamers who want an overlay showing live results, or for players who want a quick glance at their session without alt-tabbing.

## Installation

Download the installer for your platform from the [Releases](https://github.com/fohte/bms-dashtray/releases) page:

- `bms-dashtray_<version>_x64-setup.exe` — Windows
- `bms-dashtray_<version>_aarch64.dmg` — macOS (Apple Silicon)
- `bms-dashtray_<version>_x64.dmg` — macOS (Intel)
- `bms-dashtray_<version>_amd64.deb` — Linux (Debian / Ubuntu)
- `bms-dashtray_<version>_amd64.AppImage` — Linux (other distros)

The app checks for updates on launch and shows a notification bar when a new version is available. Click **"UPDATE"** to download and apply it.

### Troubleshooting

The app is not code-signed, so your OS may block it on first launch. This is expected and safe to bypass.

<details>
<summary><strong>Windows: "Windows protected your PC"</strong></summary>

Click **"More info"**, then click **"Run anyway"**.

</details>

<details>
<summary><strong>macOS: "bms-dashtray can't be opened"</strong></summary>

Run the following command in Terminal after installing:

```sh
xattr -cr /Applications/bms-dashtray.app
```

Then open the app normally.

</details>

## Usage

1. Launch bms-dashtray. On first run it asks you to pick your beatoraja root directory (the one containing the `player` folder).
2. Start playing in beatoraja. Results show up automatically.

Open the settings screen (gear icon) to tweak background transparency, font size, or the day reset time.

## Development

Requires [mise](https://mise.jdx.dev/) and [Tauri system dependencies](https://v2.tauri.app/start/prerequisites/).

Set up the development environment:

```sh
scripts/bootstrap
```

Start the app in development mode with hot-reloading:

```sh
cd backend && cargo tauri dev
```

Build installers (`.dmg`, `.msi`, etc.) locally:

```sh
cd backend && cargo tauri build
```

Installers are output to `backend/target/release/bundle/`.

Run tests:

```sh
cd backend && cargo test
```

```sh
cd frontend && pnpm run test
```

Browse UI components in Storybook:

```sh
cd frontend && pnpm run storybook
```

## License

[MIT](LICENSE)
