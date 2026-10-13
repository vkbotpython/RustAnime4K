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

# 🎬 Anime Player (Rust)

Автор Rust-порта: **avencores**.

Видеоплеер с улучшением качества рисованного изображения с помощью
высокопроизводительного алгоритма
[Anime4K](https://github.com/bloc97/Anime4K) — полный порт на Rust проекта
[Communist02/AnimePlayer](https://github.com/Communist02/AnimePlayer)
(Python + PySide6 + python-mpv, автор оригинала Denis Mazur).

Устаревшее Python-дерево (`AnimePlayer/`, исключено из git) оставлено только
как поведенческий референс и будет удалено, когда порт достигнет паритета.

## ✨ Возможности (паритет с 2.3.3 + сверх того)

- Открытие файла / URL / папки, плейлист, drag & drop (субтитры подхватываются сами,
  symlink открывается как URL), заголовок окна = имя файла
- Пресеты Anime4K: качества S/M/L/VL/UL × режимы A/B/C/A+A/B+B/C+A + набор HQ,
  генератор `mpv.conf` для Android, справка (EN/RU)
- **Диалог выбора пресета при старте видео**: выбрать пресет или полностью
  отключить; выбор **запоминается для каждого файла** (`filePresets`
  в конфиге) и молча применяется при повторном открытии. Отключить предложение
  можно в диалоге («Больше не спрашивать») или в настройках
- Воспроизведение: играть/пауза, **ползунок перемотки во всю ширину окна**
  (точность ×10, шаг 0.5 с), скорость 0.25–3.0, масштаб, громкость 100/150%,
  меню аудио- и субтитровых дорожек, автопереход к следующему файлу в конце,
  поддержка URL через `ytdl`
- Мышь: клик по видео — играть/пауза, дабл-клик — fullscreen, правый клик —
  контекстное меню. В fullscreen панели и курсор автоматически прячутся
  (если не включена фиксированная панель)
- Горячие клавиши — полный набор из оригинала (см. таблицу ниже)
- Режим SVP (`mpvpipe` + `auto-copy`), задержка звука −100…+100 с, скриншоты
- Аппаратное декодирование (`hwdec=auto-safe` с откатом на софт) — в релизных
  сборках libmpv уже включён: на Linux лежит в `lib/` рядом с бинарником,
  на Windows `libmpv-2.dll` рядом с exe
- 4 языка (EN/RU/JA/PT-BR, автоопределение + ручной выбор), 5 палитр ×
  системная/светлая/тёмная схема, 3 стиля интерфейса, вставка из буфера обмена
  (кнопки Paste) во всех диалогах
- Восстановление сессии (последний файл/URL/папка + позиция ≥ 10 с, + сохранённый
  пресет файла), параметры запуска, «О программе» с иконкой, Telegram и донатом
- Windows: без консольного окна, иконка вшита в exe (`build.rs` + winres)
  и в заголовок окна
- CLI: `anime-player <файл|папка|url>` (без восстановления сессии, как в оригинале)

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
  main.rs      CLI + запуск eframe, иконка окна  <- main.py (__main__)
  app.rs       главное окно + 9 диалогов         <- MainWindow + классы *Window
               (настройки, о программе, URL, папка, скриншот,
                параметры запуска, Android, справка + выбор пресета)
  player.rs    логика плейлиста/громкости        <- class Player
  mpv.rs       трейт MpvBackend + MockMpv        <- mpv.py (ctypes)
               + LibMpv + track-list (--features video)
  video.rs     OpenGL render ctx libmpv          <- class Video(QOpenGLWidget)
               → поверхность egui (--features video)
  config.rs    JSON-конфиг                       <- config.py
  anime4k.rs   пресеты шейдеров                  <- anime4k.py
  i18n.rs      локализация                       <- localization.py
  formats.rs   расширения медиафайлов            <- кортежи форматов в main.py
  playlist.rs  сканирование папок                <- list_files/list_filenames
  theme.rs     палитры + тема + стили egui       <- palettes.py
build.rs       иконка exe через winres (только Windows)
assets/        shaders/ images/ docs/ favicon.ico + i18n/*.json
packaging/     windows/installer.iss + linux/.desktop
.github/       workflows/ci.yml + release.yml
```

Зависимости сверх eframe/egui: `arboard` (кнопки Paste из буфера обмена),
`dark-light` (системная тема), `egui_extras/image` (PNG в «О программе»),
`winres` (иконка exe, build).

## ⚙️ Конфиг (`config.json`)

Рядом с остальными ключами (`volume`, `language`, `palette`, `style`,
`colorScheme`, `soundOffset`, `SVP`, `volumePlus`, `onOpenLastFile`,
`onPosLastFile`, `fixedControlPanel`, `opened`, `file`, `position`,
`linksHistory`, `foldersHistory`, `launchParameters`) хранятся:

- `askPresetOnOpen` — предлагать ли выбор пресета при старте видео (default `true`)
- `filePresets` — пресет для каждого файла:

```json
"filePresets": {
  "D:/anime/ep01.mkv": { "quality": "M", "mode": "A" },
  "D:/anime/ep02.mkv": { "hq": "A (Death)" },
  "D:/anime/op.mp4": { "disabled": true }
}
```

## 🔧 Сборка

Требуется стабильный Rust.

```powershell
cargo test            # 30 юнит-тестов (системные зависимости не нужны)
cargo run             # debug-сборка, mock-бэкенд (UI полностью работает, без декодирования)
cargo run -- <файл>   # открыть файл/папку/URL напрямую
```

### ▶️ С настоящим воспроизведением (фича `video`)

Релизные сборки с GitHub уже идут с фичей `video`: настоящее декодирование
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

Windows: нужна пара import-библиотека + DLL с именами `mpv.lib` / `mpv.dll`
(`libmpv2-sys` линкует `cargo:rustc-link-lib=mpv`). Официальные сборки mpv
поставляют `libmpv-2.dll`, поэтому либо сгенерируй import-библиотеку из неё
(так делает наш `release.yml`: `dumpbin /exports` → `.def` → `lib.exe`),
либо возьми пакет с `mpv.lib` (например, vcpkg), затем:

```powershell
$env:RUSTFLAGS='-L <каталог с mpv.lib>'
cargo run --features video
```

### 📦 Самодостаточность сборки

Релизные архивы не требуют ничего доустанавливать в систему (проверено:
`dumpbin /dependents`, аудит `DT_NEEDED`, CI гоняет `ldd` + запуск `--help`):

- Windows: только системные DLL (`kernel32`, `user32`, `opengl32`, …),
  CRT слинкован статически — VCRedist не нужен. Консольного окна нет
  (`windows_subsystem`), логи mpv в терминал не сыплются (`terminal=no`).
- Linux: бинарь линкуется только с glibc; X11/Wayland/OpenGL подхватываются
  в рантайме через `dlopen` — достаточно десктопной сессии с Mesa-драйверами.
  Приватные зависимости (`libmpv.so`, ffmpeg-цепочка) лежат в `lib/` рядом
  с бинарником (RPATH `$ORIGIN`).
- Windows: `libmpv-2.dll` едет в архиве и ставится инсталлятором.
- Релизы собираются с `--features video`: отдельный системный libmpv не нужен.

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
  Джоба `build-release` собирает `--features video` с бандлингом libmpv —
  её артефакты (`anime-player-Windows` / `anime-player-Linux`) и есть
  рабочий плеер, качать из вкладки Actions.
- `release.yml`: dormant, срабатывает только по тегу `v*` (сборка релизных
  бинарников + Inno Setup + публикация GitHub Release).

# 📜 Лицензия

Проект распространяется под лицензией GPL-3.0. Полный текст лицензии содержится в файле [`LICENSE`](LICENSE).

---

# 💰 Поддержать автора
+ **SBER**: `2202 2050 1464 4675`