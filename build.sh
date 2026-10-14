#!/bin/bash
# Локальная сборка Anime4K Player (Linux): проверки + release + dist-пакет.
# Повторяет шаги CI (ci.yml / release.yml) локально.
#
#   ./build.sh                    # всё сразу (нужен libmpv-dev; для пакета — patchelf)
#   ./build.sh --no-video         # без libmpv (только mock-бэкенд)
#   ./build.sh --skip-checks       # пропустить fmt/clippy/test
#   ./build.sh --no-package       # только target/release без dist
#   ./build.sh --installer        # + install.sh в пакете: согласие с LICENSE,
#                               #   выбор каталога, ярлык в меню, ссылка в ~/.local/bin
set -euo pipefail

NO_VIDEO=0
SKIP_CHECKS=0
NO_PACKAGE=0
INSTALLER=0
for arg in "$@"; do
  case "$arg" in
    --no-video) NO_VIDEO=1 ;;
    --skip-checks) SKIP_CHECKS=1 ;;
    --no-package) NO_PACKAGE=1 ;;
    --installer) INSTALLER=1 ;;
    -h|--help)
      sed -n '2,10p' "$0"
      exit 0
      ;;
    *) echo "Неизвестный аргумент: $arg (см. --help)"; exit 1 ;;
  esac
done

cd "$(dirname "$0")"
step() { echo; echo "=== $1 ==="; }

command -v cargo >/dev/null || { echo 'Нужен Rust: https://rustup.rs'; exit 1; }

FEATURES=()
if [ "$NO_VIDEO" -eq 0 ]; then
  FEATURES=(--features video)
  if ! pkg-config --exists mpv 2>/dev/null; then
    echo 'Нужен libmpv-dev. Например (как в CI):'
    echo '  sudo apt install libmpv-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libxkbcommon-x11-dev libgl1-mesa-dev patchelf'
    echo 'Либо собери без video: ./build.sh --no-video'
    exit 1
  fi
fi

# --- 1. Проверки (как lint-test в CI) ---
if [ "$SKIP_CHECKS" -eq 0 ]; then
  step 'cargo fmt --check'
  cargo fmt --all -- --check
  step 'cargo clippy'
  cargo clippy --all-targets -- -D warnings
  if [ "$NO_VIDEO" -eq 0 ]; then
    step 'cargo clippy --features video'
    cargo clippy --all-targets --features video -- -D warnings
  fi
  step 'cargo test'
  cargo test "${FEATURES[@]}"
else
  echo 'Проверки пропущены (--skip-checks)'
fi

# --- 2. Release-сборка ---
step 'cargo build --release'
cargo build --release "${FEATURES[@]}"

# Шейдеры рядом с exe: glsl-shaders резолвится от каталога exe, без них
# пресеты Anime4K молча не применяются при запуске из target/release.
if [ "$NO_VIDEO" -eq 0 ]; then
  mkdir -p target/release/shaders
  cp -r assets/shaders/. target/release/shaders/
fi

# --- 3. Smoke-тест ---
step 'smoke-test: anime4k-player --help'
./target/release/anime4k-player --help >/dev/null

# --- 4. Dist-пакет (раскладка 1:1 с release.yml) ---
if [ "$NO_PACKAGE" -eq 0 ]; then
  command -v patchelf >/dev/null || { echo 'Для пакета нужен patchelf (или используй --no-package)'; exit 1; }
  step 'Сборка dist/anime4k-player'
  rm -rf dist
  mkdir -p dist/anime4k-player/lib
  cp target/release/anime4k-player dist/anime4k-player/
  # Только рантайм-зависимости: шейдеры резолвятся от каталога exe.
  # images/ (иконка вшита в бинарь), docs/ (справка встроена в код)
  # и favicon.ico (нужен только build.rs и инсталлятору) не кладём.
  cp -r assets/shaders dist/anime4k-player/
  cp packaging/linux/*.desktop README.md LICENSE dist/anime4k-player/
  if [ "$INSTALLER" -eq 1 ]; then
    step 'Генерация dist/anime4k-player/install.sh'
    cat > dist/anime4k-player/install.sh <<'INSTALLER_EOF'
#!/bin/bash
# Anime4K Player — интерактивный установщик для Linux.
# Показывает LICENSE и просит согласие, спрашивает каталог установки,
# копирует файлы, ставит .desktop-ярлык и ссылку в ~/.local/bin.
set -euo pipefail
cd "$(dirname "$0")"

echo '=== Лицензия (GPL-3.0-or-later) ==='
if command -v less >/dev/null 2>&1; then
  less LICENSE
else
  cat LICENSE
fi
read -rp 'Принимаете условия лицензии? [y/N] ' answer
case "$answer" in
  [yY]|[yY][eE][sS]|[dD]|[дД]*) ;;
  *) echo 'Установка отменена.'; exit 1 ;;
esac

default_dir="$HOME/.local/anime4k-player"
read -rp "Каталог установки [$default_dir]: " prefix
prefix="${prefix:-$default_dir}"
case "$prefix" in
  "$PWD"|"$PWD"/*) echo "Нельзя устанавливать внутрь самого пакета ($PWD). Выберите другой каталог."; exit 1 ;;
esac
mkdir -p "$prefix"
for f in *; do
  [ "$f" = install.sh ] || cp -r "$f" "$prefix/"
done

read -rp 'Создать ярлык в меню приложений? [Y/n] ' shortcut
case "$shortcut" in
  [nN]|[nN][oO]|[нН]*) echo 'Ярлык пропущен.' ;;
  *)
    apps_dir="$HOME/.local/share/applications"
    mkdir -p "$apps_dir"
    sed -e "s|^Exec=.*|Exec=$prefix/anime4k-player|" \
        -e "s|^Icon=.*|Icon=$prefix/favicon.ico|" \
        io.github.avencores.Anime4KPlayer.desktop > "$apps_dir/io.github.avencores.Anime4KPlayer.desktop"
    echo "Ярлык: $apps_dir/io.github.avencores.Anime4KPlayer.desktop"
    ;;
esac

read -rp 'Добавить запуск через ~/.local/bin/anime4k-player? [Y/n] ' add_bin
case "$add_bin" in
  [nN]|[nN][oO]|[нН]*) echo 'Ссылка пропущена.' ;;
  *)
    mkdir -p "$HOME/.local/bin"
    ln -sf "$prefix/anime4k-player" "$HOME/.local/bin/anime4k-player"
    echo 'Ссылка: ~/.local/bin/anime4k-player (проверьте, что ~/.local/bin в PATH)'
    ;;
esac

echo "Готово: Anime4K Player установлен в $prefix"
INSTALLER_EOF
    chmod +x dist/anime4k-player/install.sh
    echo "Установщик: $PWD/dist/anime4k-player/install.sh (войдёт и в tar-архив)"
  fi
  if [ "$NO_VIDEO" -eq 0 ]; then
    # Bundle libmpv.so + приватные зависимости (цепочка ffmpeg, ...).
    # Хостовые стеки (glibc, X11/Wayland, Mesa/GL, DRM) исключаются.
    EXCLUDE='^lib(c|m|dl|pthread|rt|resolv)\.so|ld-linux|libGL\.|libEGL\.|libGLX|libGLdispatch|libOpenGL|libX11\.|libxcb\.|libXau\.|libXdmcp\.|libxkb|libwayland|libdrm\.|libvulkan\.|libgbm\.'
    QUEUE=()
    while read -r line; do QUEUE+=("$line"); done < <(ldd target/release/anime4k-player | grep -oP '=> \K/\S+')
    declare -A SEEN=()
    while [ "${#QUEUE[@]}" -gt 0 ]; do
      LIB="${QUEUE[0]}"; QUEUE=("${QUEUE[@]:1}")
      BASE="$(basename "$LIB")"
      [ -n "${SEEN[$BASE]:-}" ] && continue
      SEEN[$BASE]=1
      echo "$BASE" | grep -Eq "$EXCLUDE" && continue
      cp -L "$LIB" "dist/anime4k-player/lib/$BASE"
      while read -r DEP; do
        [ -n "$DEP" ] && QUEUE+=("$DEP")
      done < <(ldd "$LIB" 2>/dev/null | grep -oP '=> \K/\S+')
    done
    patchelf --set-rpath '$ORIGIN/lib' --force-rpath dist/anime4k-player/anime4k-player
    for F in dist/anime4k-player/lib/*.so*; do
      patchelf --set-rpath '$ORIGIN' --force-rpath "$F" 2>/dev/null || true
    done
    echo '--- bundled libs ---'
    ls dist/anime4k-player/lib
    echo '--- ldd shipped binary ---'
    ldd dist/anime4k-player/anime4k-player
    if ldd dist/anime4k-player/anime4k-player | grep -q 'not found'; then
      echo 'ERROR: missing shared libraries'; exit 1
    fi
    tar -czf anime4k-player-linux-x86_64.tar.gz -C dist anime4k-player
    echo "Архив: $PWD/anime4k-player-linux-x86_64.tar.gz"
  fi
  echo "Готово: $PWD/dist/anime4k-player"
else
  if [ "$INSTALLER" -eq 1 ]; then
    echo '--installer требует пакета (уберите --no-package)'; exit 1
  fi
  echo 'Пакет пропущен (--no-package). Бинарь: target/release/anime4k-player'
fi

echo
echo 'Сборка завершена успешно.'
