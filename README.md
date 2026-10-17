<div align="center">
    <a href="https://www.youtube.com/@avencores/" target="_blank">
      <img src="https://github.com/user-attachments/assets/338bcd74-e3c3-4700-87ab-7985058bd17e" alt="YouTube" height="40">
    </a>
    <a href="https://t.me/avencoresyt" target="_blank">
      <img src="https://github.com/user-attachments/assets/939f8beb-a49a-48cf-89b9-d610ee5c4b26" alt="Telegram" height="40">
    </a>
    <a href="https://vk.ru/avencoresreuploads" target="_blank">
      <img src="https://github.com/user-attachments/assets/dc109dda-9045-4a06-95a5-3399f0e21dc4" alt="VK" height="40">
    </a>
    <a href="https://dzen.ru/avencores" target="_blank">
      <img src="https://github.com/user-attachments/assets/bd55f5cf-963c-4eb8-9029-7b80c8c11411" alt="Dzen" height="40">
    </a>
</div>

# 🎬 Anime4K Player

![1](https://i.ibb.co/3m4gp6dK/1.png)

Видеоплеер с улучшением качества рисованного изображения с помощью
высокопроизводительного алгоритма
[Anime4K](https://github.com/bloc97/Anime4K), написанный на Rust
(egui + libmpv).

## ✨ Возможности

- Открытие файла / URL / папки, плейлист, drag & drop (субтитры подхватываются сами,
  symlink открывается как URL), заголовок окна = `имя - Anime4K Player`
- Пресеты Anime4K: качества S/M/L/VL/UL × режимы A/B/C/A+A/B+B/C+A + набор HQ,
  генератор строки `glsl-shaders=` для Android, структурированная справка (EN/RU):
  карточки режимов, таблицы конвейеров и шейдеров, рекомендации
- **Диалог выбора пресета при старте видео**: выбрать пресет или полностью
  отключить; выбор **запоминается для каждого файла** (`filePresets`
  в конфиге) и молча применяется при повторном открытии. Отключить предложение
  можно в диалоге («Больше не спрашивать») или в настройках
- Воспроизведение: играть/пауза, **ползунок перемотки во всю ширину окна**
  (точность 0.1 с), скорость 0.25–3.0 (12 ступеней в меню), масштаб ±0.01,
  громкость 100/150%, меню аудио- и субтитровых дорожек + подключение внешних
  файлов, строка медиа-инфо (пресет, кодек, разрешение, FPS, потери кадров,
  HW-декодер, живые метрики UI/V и CACHE-WAIT), автопереход к следующему
  файлу в конце, поддержка URL через `ytdl`
- Мышь: клик по видео — играть/пауза, дабл-клик — fullscreen, правый клик —
  контекстное меню. В fullscreen панели и курсор автоматически прячутся
  (если не включена фиксированная панель)
- Горячие клавиши (см. таблицу ниже)
- Режим SVP (`mpvpipe` + `auto-copy`), задержка звука −100…+100 с (шаг 0.01),
  скриншоты JPEG в выбранную папку (q90 по умолчанию; q100 — параметром
  запуска `screenshot-jpeg-quality`), диагностика `mpv.log` рядом с exe
  (ротация 512 КБ)
- Аппаратное декодирование (`hwdec=auto-safe` с откатом на софт) — в релизных
  сборках libmpv уже включён: на Linux лежит в `lib/` рядом с бинарником,
  на Windows `libmpv-2.dll` рядом с exe
- 25 языков (автоопределение + ручной выбор), 5 палитр ×
  системная/светлая/тёмная схема, 3 стиля интерфейса, вставка из буфера обмена
  (кнопки Paste) во всех диалогах
- Фон окна **Mica / Mica Alt / Acrylic** (Windows 11 22H2+): полупрозрачные
  панели и диалоги, сквозь которые просвечивают обои рабочего стола
  (Настройки → «Фон окна», по умолчанию выключено)
- Восстановление сессии (последний файл/URL/папка + позиция ≥ 10 с, сохранённый
  пресет файла, геометрия окна, громкость, SVP/сдвиг звука/параметры запуска),
  параметры запуска, «О программе» с иконкой, чипами стека,
  соцсетями (YouTube/Telegram/VK/Дзен/GitLab) и донатом с подтверждением копирования
- Windows: без консольного окна, иконка вшита в exe (`build.rs` + winres)
  и в заголовок окна
- CLI: `anime4k-player <файл|папка|url>` (без восстановления сессии),
  `--help`/`--version` — справка и версия

## ⌨️ Горячие клавиши

| Клавиши | Действие |
|---|---|
| `Ctrl+O` / `Ctrl+U` / `F2` | Открыть файл / URL / папку |
| `F4` | Закрыть текущее |
| `F5` / `F1` / `F6` | Настройки / О программе / плейлист |
| `F11` / `F12` | Fullscreen / скриншот |
| `Space` | Играть / пауза |
| `D` | Отключить Anime4K (запоминается за файлом) |
| `0` / `1` / `2` / `3` | Скорость x0.25 / x1.0 / x2.0 / x3.0 |
| `←` / `→` | Перемотка ∓5 с |
| `↑` / `↓` | Громкость ±5 |
| `+` / `-` | Масштаб видео ±0.01 |
| `Esc` | Выйти из fullscreen / закрыть диалоги |
| `Alt+F4` | Выход |

## 🗂️ Структура

```text
src/
  main.rs      CLI + запуск eframe, иконка окна
  app/         главное окно и диалоги
    mod.rs       eframe::App (цикл ui/on_exit) + реэкспорты
    state.rs     Anime4KPlayerApp, снапшот mpv, выбор пресета, new()
    helpers.rs   чистые функции: время, пути, диалоги выбора файлов
    playback.rs  открытие, пресеты Anime4K, опрос mpv
    session.rs   восстановление сессии и сохранение при выходе
    input.rs     горячие клавиши, мышь, fullscreen, drag & drop
    views.rs     меню, видео, транспорт, треки, плейлист
    dialogs/    9 диалоговых окон в 8 модулях + mod/ui_kit: settings, about,
                open (URL + папка — 2 окна), screenshot, launch, android,
                help, preset_offer; ui_kit — общие карточки/плашки диалогов
  player.rs    логика плейлиста/громкости
  mpv/         управление mpv
    mod.rs       трейт MpvBackend + Track/TrackKind
    mock.rs      MockMpv для тестов и сборок без video
    libmpv.rs    LibMpv + track-list + mpv.log (--features video)
    files.rs     поиск новых файлов (скриншоты)
  video.rs     OpenGL render ctx libmpv
               → поверхность egui (--features video)
  config.rs    JSON-конфиг
  anime4k.rs   пресеты шейдеров
  i18n.rs      локализация
  formats.rs   расширения медиафайлов
  playlist.rs  сканирование папок
  icon.rs      PNG-иконка приложения (окно + «О программе»)
  fonts.rs     фоллбэки шрифтов (Hack для стрелок/символов)
  reference/   справка Anime4K (EN/RU)
    mod.rs       реэкспорты + точка входа справки
    view.rs      секции: шапка, выбор, режимы, тонкая настройка, советы
    data.rs      режимы, шейдеры, конвейеры
    widgets.rs   карточки, бейджи, секции
  theme.rs     палитры + тема + стили egui
  mica.rs      фон окна Mica/Acrylic (Windows 11, DWM)
build.rs       иконка exe через winres (только Windows)
build.ps1      локальная сборка Windows (проверки + release + dist + -Installer/-Zip)
build.sh       локальная сборка Linux (проверки + release + dist + .deb/.rpm)
assets/        shaders/ (49 .glsl) + images/ (иконка, play, icons/*.svg) +
               docs/ (2 .txt) + favicon.ico + i18n/*.json (25 языков)
packaging/     windows/installer.iss (Inno Setup: лицензия, каталог, ярлыки) +
               linux/{io.github.avencores.Anime4KPlayer.desktop, mkpackages.sh (.deb/.rpm)}
.cargo/config.toml  флаги `-L mpvlib-local` для линковки libmpv — единственный
                    источник у скриптов и ручных вызовов cargo (каталог
                    локальный, в .gitignore: mpv.lib генерируется из DLL)
.github/       workflows/ci.yml + release.yml
```

Зависимости сверх eframe/egui: `serde`/`serde_json` (JSON-конфиг), `dirs`
(пути конфига), `anyhow` (ошибки), `rfd` (нативные диалоги файлов),
`sys-locale` (язык системы), `arboard` (кнопки Paste из буфера обмена),
`dark-light` (системная тема), `egui_extras/image` (PNG в «О программе»),
`raw-window-handle` (HWND окна для DWM), `windows-sys` (Mica/Acrylic-фон),
`winres` (иконка exe, build), `tempfile` (dev, тесты).

## ⚙️ Конфиг (`config.json`)

Лежит в `%APPDATA%/Anime4K Player/config.json` (Windows) или
`~/.config/Anime4K Player/config.json` (Linux).

Рядом с остальными ключами (`volume`, `language`, `palette`, `style`,
`colorScheme`, `backdrop`, `soundOffset`, `SVP`, `volumePlus`,
`onOpenLastFile`,
`onPosLastFile`, `fixedControlPanel`, `opened`, `file`, `position`,
`linksHistory`, `foldersHistory`, `launchParameters`, `screenshotDirectory`,
`androidPath`, `windowX`, `windowY`, `windowW`, `windowH`, `windowMaximized`)
хранятся:

- `askPresetOnOpen` — предлагать ли выбор пресета при старте видео (default `true`)
- `filePresets` — пресет для каждого файла (хранятся последние 500):

```json
"filePresets": {
  "D:/anime/ep01.mkv": { "quality": "M", "mode": "A" },
  "D:/anime/ep02.mkv": { "hq": "A (Death)" },
  "D:/anime/op.mp4": { "disabled": true }
}
```

## 🔧 Сборка

Требуется стабильный Rust.

### 🧱 Локальная сборка скриптом

Скрипты повторяют шаги CI 1:1 (проверки → release → smoke-тест → `dist`-пакет):

```powershell
.\build.ps1              # всё сразу: проверки + release с video + dist-пакет
.\build.ps1 -NoVideo     # без libmpv (только mock-бэкенд, DLL не нужна)
.\build.ps1 -SkipChecks  # пропустить fmt/clippy/test
.\build.ps1 -NoPackage   # только target/release без dist
.\build.ps1 -Zip         # + zip-архив как в CI
.\build.ps1 -Installer   # + Inno Setup-установщик (anime4k-player-windows-x86_64-setup.exe):
                         #   согласие с LICENSE, выбор каталога, меню «Пуск», ярлык,
                         #   ассоциации медиафайлов. Нужен Inno Setup 6:
                         #   winget install -e --id JRSoftware.InnoSetup
.\build.ps1 -Zip -Installer # портативный архив и установщик рядом
.\build.ps1 -DownloadMpv # скачать mpv-dev и собрать mpv.lib (нужны VS BuildTools + 7z)
```

```bash
./build.sh              # всё сразу (нужен libmpv-dev + xcb/xkb/GL-dev, см. build.sh;
                        #   для пакета — patchelf, dpkg-deb и rpmbuild: sudo apt install rpm)
./build.sh --no-video --no-package
```

Пакет Linux — системные `anime4k-player-linux-x86_64.deb` (apt) и
`anime4k-player-linux-x86_64.rpm` (dnf) в корне репо: `/opt/anime4k-player`
+ симлинк `/usr/bin/anime4k-player` + ярлык + иконка. Установка:

```bash
sudo apt install ./anime4k-player-linux-x86_64.deb  # Debian/Ubuntu
sudo dnf install ./anime4k-player-linux-x86_64.rpm  # Fedora
```

Windows, фича `video`: для линковки используется `mpvlib-local/mpv.lib`
(путь `-L` задан в `.cargo/config.toml`, поэтому одинаков у скриптов и у
ручных вызовов cargo — это сохраняет общий кэш в `target/`; если `mpv.lib`
ещё нет, `build.ps1` соберёт его из найденной DLL через `dumpbin`/`lib.exe`),
а `libmpv-2.dll` ищется по `-MpvDll <путь>`, в корне репо, `mpvlib-local/`,
`target/release/`, `target/debug/`, `dist/anime4k-player/`, установленных
`mpv`/`mpv.net` и каталогах `PATH`. Готовый пакет лежит в
`dist/anime4k-player/` — его можно запускать прямо оттуда.

```powershell
cargo test                   # 57 юнит-тестов, mock-бэкенд (системные зависимости не нужны)
cargo test --features video  # 58 тестов (линковка через mpvlib-local/mpv.lib)
cargo run                    # debug-сборка, mock-бэкенд (UI полностью работает, без декодирования)
cargo run -- <файл>          # открыть файл/папку/URL напрямую
```

### ▶️ С настоящим воспроизведением (фича `video`)

Релизные сборки уже идут с фичей `video`: настоящее декодирование
через `libmpv` с аппаратным ускорением (`hwdec=auto-safe`), рендер через
OpenGL render-контекст на поверхность egui. Библиотека едет вместе с плеером:
на Linux — `libmpv.so` и ffmpeg-цепочка в каталоге `lib/` (RPATH `$ORIGIN`),
на Windows — `libmpv-2.dll` рядом с exe. От системы нужны только десктопная
сессия с Mesa-драйверами (Linux) — ничего доустанавливать не надо.

Вручную (для разработки) фича собирается так.

Linux:

```bash
sudo apt install libmpv-dev
cargo run --features video
```

Windows: нужны import-библиотека (`libmpv2-sys` линкует
`cargo:rustc-link-lib=mpv`) и `libmpv-2.dll` для запуска. Официальные
сборки mpv поставляют только DLL, поэтому `mpv.lib` генерируется из неё
(`dumpbin /exports` → `.def` → `lib.exe`) — так делают `release.yml` и
`build.ps1`. Положи `mpv.lib` в `mpvlib-local/` (этот путь уже прописан в
`.cargo/config.toml`, ничего дополнительно выставлять не нужно) — либо
просто запусти `.\build.ps1`: он найдёт DLL и соберёт `mpv.lib` сам
(`-DownloadMpv` — ещё и скачает mpv-dev). Затем:

```powershell
cargo run --features video
```

### 📦 Самодостаточность сборки

Релизные пакеты не требуют ничего доустанавливать в систему (проверено:
`dumpbin /dependents`, аудит `DT_NEEDED`, CI гоняет `ldd` + запуск `--help`):

- Windows: только системные DLL (`kernel32`, `user32`, `opengl32`, …),
  CRT слинкован статически — VCRedist не нужен. Консольного окна нет
  (`windows_subsystem`), логи mpv в терминал не сыплются (`terminal=no`).
- Linux: бинарь линкуется только с glibc; X11/Wayland/OpenGL подхватываются
  в рантайме через `dlopen` — достаточно десктопной сессии с Mesa-драйверами.
  Приватные зависимости (`libmpv.so`, ffmpeg-цепочка) лежат в `lib/` рядом
  с бинарником (`/opt/anime4k-player/lib`, RPATH `$ORIGIN`).
- Windows: `libmpv-2.dll` едет в архиве и ставится инсталлятором.
- Релизы собираются с `--features video`: отдельный системный libmpv не нужен.

### ⚙️ Безопасные параметры запуска

Сохранённые параметры используют
безопасное подмножество `имя=значение` (по одному на строку, комментарии `#`),
применяются через `mpv.set_property`. Пример:

```text
volume=80
hwdec=auto
audio-delay=0.15
```

## 🚀 CI / релизы

- `ci.yml`: `fmt --check` + `clippy -D warnings` + `cargo check/clippy --features video`
  + `cargo test` на Windows + Linux; плюс `cargo test --features video` на Linux.
  Джоба `build-release` собирает `--features video` с бандлингом libmpv —
  её артефакты и есть рабочий плеер, качать из вкладки Actions:
  Windows — два отдельных артефакта: `anime4k-player-Windows-portable`
  (`anime4k-player-windows-x86_64.zip`) и `anime4k-player-Windows-setup`
  (`anime4k-player-windows-x86_64-setup.exe`, визард: согласие
  с лицензией, выбор каталога, ярлыки, ассоциации файлов);
  Linux — два отдельных артефакта: `anime4k-player-Linux-deb`
  (`anime4k-player-linux-x86_64.deb`, `sudo apt install ./....deb`)
  и `anime4k-player-Linux-rpm` (`anime4k-player-linux-x86_64.rpm`,
  `sudo dnf install ./....rpm`).
- `release.yml`: dormant, срабатывает только по тегу `v*` (сборка релизных
  бинарников + Inno Setup + публикация релиза).

# 📜 Лицензия

Проект распространяется под лицензией GPL-3.0. Полный текст лицензии содержится в файле [`LICENSE`](LICENSE).

---

# 💰 Поддержать автора
+ **SBER**: `2202 2050 1464 4675`