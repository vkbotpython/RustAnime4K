# Anime Player (Rust)

Video player with hand-drawn image upscaling via the high-performance
[Anime4K](https://github.com/bloc97/Anime4K) algorithm — a complete Rust port
of [Communist02/AnimePlayer](https://github.com/Communist02/AnimePlayer)
(Python + PySide6 + python-mpv).

The legacy Python tree (`AnimePlayer/`, git-ignored) is kept only as a
behavioral reference and will be removed once the port reaches parity.

![status](https://github.com/Communist02/AnimePlayer/actions/workflows/ci.yml/badge.svg)

## Features (parity with 2.3.3)

- Open file / URL / folder, playlist, drag & drop (subtitle files auto-attach)
- Anime4K presets: qualities S/M/L/VL/UL × modes A/B/C/A+A/B+B/C+A + HQ set,
  Android `mpv.conf` generator, reference docs (EN/RU)
- Playback: play/pause, seek slider (×10 granularity), speed 0.25–3.0,
  zoom, volume 100/150%, audio/subtitle track menus
- SVP mode (`mpvpipe` + `auto-copy`), sound offset, screenshots
- 4 locales (EN/RU/JA/PT-BR, auto-detect + override), 5 palettes × light/dark
- Session restore (last file/URL/folder + position ≥ 10 s), launch parameters
- CLI: `anime-player <file|dir|url>` (skips session restore, like the original)

## Layout

```text
src/
  main.rs      CLI + eframe bootstrap        <- main.py (__main__)
  app.rs       main window + 9 dialogs        <- MainWindow + *Window classes
  player.rs    playlist/current/volume logic  <- class Player
  mpv.rs       MpvBackend trait + MockMpv     <- mpv.py (ctypes)
               + LibMpv (--features video)
  config.rs    JSON config                    <- config.py
  anime4k.rs   shader presets                 <- anime4k.py
  i18n.rs      locales                        <- localization.py
  formats.rs   media extensions               <- format tuples in main.py
  playlist.rs  folder scanning                <- list_files/list_filenames
  theme.rs     egui palettes                  <- palettes.py
assets/        shaders/ images/ docs/ favicon.ico + i18n/*.json
packaging/     windows/installer.iss + linux/.desktop
.github/       workflows/ci.yml + release.yml
```

## Build

Prerequisites: stable Rust.

```powershell
cargo test            # 26 unit tests (no system deps needed)
cargo run             # debug build, mock backend (UI fully usable, no decoding)
cargo run -- <file>   # open a file/folder/URL directly
```

### With real playback (`video` feature)

Uses `libmpv` directly (same embedding model as the Python build:
`keep-open`, `gpu-hq` profile, `glsl-shaders` for Anime4K).

Linux:

```bash
sudo apt install libmpv-dev
cargo run --features video
```

Windows: install mpv development files and put `mpv-2.dll` next to the
binary (shipped separately, as with the Python releases), then:

```powershell
cargo run --features video
```

> Video rendering note: with `video` enabled you get real audio + full mpv
> property control today; the OpenGL render-context widget inside the egui
> surface (the equivalent of the old `Video(QOpenGLWidget)` class, same
> approach as `egui-sharkplayer`) is the next milestone — the surface
> currently shows a placeholder.

### Safe launch parameters

The old `exec(...)` box is gone by design. Stored parameters use a safe
`name=value` subset (one per line, `#` comments), applied via
`mpv.set_property`. Example:

```text
volume=80
hwdec=auto
audio-delay=0.15
```

## CI / releases

- `ci.yml`: `fmt --check` + `clippy -D warnings` + `cargo test` on
  Windows + Linux; plus `cargo check/test --features video` on Linux.
- `release.yml`: on tag `v*` builds release binaries for Windows + Linux,
  builds the Inno Setup installer on Windows, and publishes a GitHub Release
  with portable archives.

## Roadmap

1. ✅ Scaffold: logic/UI/CI in repo root (this state)
2. Render-context video widget (`libmpv` OpenGL → egui surface)
3. Native file dialogs, tray/fullscreen cursor polish, file associations
4. mpv track-list parsing for audio/sub menus under `video`
5. Delete `AnimePlayer/` reference tree

## License

GPL-3.0-or-later, same as the original. See `LICENSE`.
Original project © Denis Mazur; shaders © bloc97/Anime4K contributors.
