# 🎬 Anime Player (Rust)

Автор Rust-порта: **avencores**.

Видеоплеер с улучшением качества рисованного изображения с помощью
высокопроизводительного алгоритма
[Anime4K](https://github.com/bloc97/Anime4K) — полный порт на Rust проекта
[Communist02/AnimePlayer](https://github.com/Communist02/AnimePlayer)
(Python + PySide6 + python-mpv, автор оригинала Denis Mazur).

Устаревшее Python-дерево (`AnimePlayer/`, исключено из git) оставлено только
как поведенческий референс и будет удалено, когда порт достигнет паритета.

## ✨ Возможности (паритет с 2.3.3)

- Открытие файла / URL / папки, плейлист, drag & drop (субтитры подхватываются сами)
- Пресеты Anime4K: качества S/M/L/VL/UL × режимы A/B/C/A+A/B+B/C+A + набор HQ,
  генератор `mpv.conf` для Android, справка (EN/RU)
- Воспроизведение: играть/пауза, ползунок перемотки (точность ×10), скорость 0.25–3.0,
  масштаб, громкость 100/150%, меню аудио- и субтитровых дорожек
- Режим SVP (`mpvpipe` + `auto-copy`), задержка звука, скриншоты
- 4 языка (EN/RU/JA/PT-BR, автоопределение + ручной выбор), 5 палитр × светлая/тёмная
- Восстановление сессии (последний файл/URL/папка + позиция ≥ 10 с), параметры запуска
- CLI: `anime-player <файл|папка|url>` (без восстановления сессии, как в оригинале)

## 🗂️ Структура

```text
src/
  main.rs      CLI + запуск eframe              <- main.py (__main__)
  app.rs       главное окно + 9 диалогов        <- MainWindow + классы *Window
  player.rs    логика плейлиста/громкости       <- class Player
  mpv.rs       трейт MpvBackend + MockMpv       <- mpv.py (ctypes)
               + LibMpv + track-list (--features video)
  video.rs     OpenGL render ctx libmpv         <- class Video(QOpenGLWidget)
               → поверхность egui (--features video)
  config.rs    JSON-конфиг                      <- config.py
  anime4k.rs   пресеты шейдеров                 <- anime4k.py
  i18n.rs      локализация                      <- localization.py
  formats.rs   расширения медиафайлов           <- кортежи форматов в main.py
  playlist.rs  сканирование папок               <- list_files/list_filenames
  theme.rs     палитры egui                     <- palettes.py
assets/        shaders/ images/ docs/ favicon.ico + i18n/*.json
packaging/     windows/installer.iss + linux/.desktop
.github/       workflows/ci.yml + release.yml
```

## 🔧 Сборка

Требуется стабильный Rust.

```powershell
cargo test            # 26 юнит-тестов (системные зависимости не нужны)
cargo run             # debug-сборка, mock-бэкенд (UI полностью работает, без декодирования)
cargo run -- <файл>   # открыть файл/папку/URL напрямую
```

### ▶️ С настоящим воспроизведением (фича `video`)

Напрямую используется `libmpv` (та же модель встраивания, что в Python-сборке:
профиль `keep-open`, `gpu-hq`, `glsl-shaders` для Anime4K, `vo=libmpv` с OpenGL
render-контекстом, который блитится на поверхность egui — эквивалент старого
класса `Video(QOpenGLWidget)`).

Linux:

```bash
sudo apt install libmpv-dev
cargo run --features video
```

Windows: нужна пара import-библиотека + DLL с именами `mpv.lib` / `mpv.dll`
(`libmpv2-sys` линкует `cargo:rustc-link-lib=mpv`). Официальные сборки mpv
поставляют `libmpv-2.dll`, поэтому либо сгенерируй import-библиотеку из неё,
либо возьми пакет с `mpv.lib` (например, vcpkg), затем:

```powershell
cargo run --features video
```

### ⚙️ Безопасные параметры запуска

Старое окно с `exec(...)` убрано сознательно. Сохранённые параметры используют
безопасное подмножество `имя=значение` (по одному на строку, комментарии `#`),
применяются через `mpv.set_property`. Пример:

```text
volume=80
hwdec=auto
audio-delay=0.15
```

## 🚀 CI / релизы

- `ci.yml`: `fmt --check` + `clippy -D warnings` + `cargo test` на
  Windows + Linux; плюс `cargo check/test --features video` на Linux.
- `release.yml`: по тегу `v*` собирает релизные бинарники для Windows + Linux,
  собирает установщик Inno Setup на Windows и публикует GitHub Release
  с portable-архивами.

# 📜 Лицензия

Проект распространяется под лицензией GPL-3.0. Полный текст лицензии содержится в файле [`LICENSE`](LICENSE).

---
# 💰 Поддержать автора
+ **SBER**: `2202 2050 1464 4675`