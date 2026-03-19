# bms-dashtray

A real-time play history dashboard for [beatoraja](https://github.com/exch-bms2/beatoraja).

bms-dashtray sits alongside beatoraja and watches its database files. Every time you finish a song, the result pops up instantly — clear lamp, EX score diff, BP diff, and whether you set a new personal best. No manual refresh, no browser tab, just a lightweight always-on-top window.

Great for streamers who want an overlay showing live results, or for players who want a quick glance at their session without alt-tabbing.

## Installation

Download the latest installer for your OS from the [Releases](https://github.com/fohte/bms-dashtray/releases) page.

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
cd frontend && pnpm run test
```

Browse UI components in Storybook:

```sh
cd frontend && pnpm run storybook
```

## License

[MIT](LICENSE)
