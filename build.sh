#!/bin/bash
# Локальная сборка Anime Player (Linux): проверки + release + dist-пакет.
# Повторяет шаги CI (ci.yml / release.yml) локально.
#
#   ./build.sh                    # всё сразу (нужен libmpv-dev; для пакета — patchelf)
#   ./build.sh --no-video         # без libmpv (только mock-бэкенд)
#   ./build.sh --skip-checks       # пропустить fmt/clippy/test
#   ./build.sh --no-package       # только target/release без dist
set -euo pipefail

NO_VIDEO=0
SKIP_CHECKS=0
NO_PACKAGE=0
for arg in "$@"; do
  case "$arg" in
    --no-video) NO_VIDEO=1 ;;
    --skip-checks) SKIP_CHECKS=1 ;;
    --no-package) NO_PACKAGE=1 ;;
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

# --- 3. Smoke-тест ---
step 'smoke-test: anime-player --help'
./target/release/anime-player --help >/dev/null

# --- 4. Dist-пакет (раскладка 1:1 с release.yml) ---
if [ "$NO_PACKAGE" -eq 0 ]; then
  command -v patchelf >/dev/null || { echo 'Для пакета нужен patchelf (или используй --no-package)'; exit 1; }
  step 'Сборка dist/anime-player'
  rm -rf dist
  mkdir -p dist/anime-player/lib
  cp target/release/anime-player dist/anime-player/
  cp -r assets/shaders assets/images assets/docs dist/anime-player/
  cp assets/favicon.ico packaging/linux/*.desktop README.md LICENSE dist/anime-player/
  if [ "$NO_VIDEO" -eq 0 ]; then
    # Bundle libmpv.so + приватные зависимости (цепочка ffmpeg, ...).
    # Хостовые стеки (glibc, X11/Wayland, Mesa/GL, DRM) исключаются.
    EXCLUDE='^lib(c|m|dl|pthread|rt|resolv)\.so|ld-linux|libGL\.|libEGL\.|libGLX|libGLdispatch|libOpenGL|libX11\.|libxcb\.|libXau\.|libXdmcp\.|libxkb|libwayland|libdrm\.|libvulkan\.|libgbm\.'
    QUEUE=()
    while read -r line; do QUEUE+=("$line"); done < <(ldd target/release/anime-player | grep -oP '=> \K/\S+')
    declare -A SEEN=()
    while [ "${#QUEUE[@]}" -gt 0 ]; do
      LIB="${QUEUE[0]}"; QUEUE=("${QUEUE[@]:1}")
      BASE="$(basename "$LIB")"
      [ -n "${SEEN[$BASE]:-}" ] && continue
      SEEN[$BASE]=1
      echo "$BASE" | grep -Eq "$EXCLUDE" && continue
      cp -L "$LIB" "dist/anime-player/lib/$BASE"
      while read -r DEP; do
        [ -n "$DEP" ] && QUEUE+=("$DEP")
      done < <(ldd "$LIB" 2>/dev/null | grep -oP '=> \K/\S+')
    done
    patchelf --set-rpath '$ORIGIN/lib' --force-rpath dist/anime-player/anime-player
    for F in dist/anime-player/lib/*.so*; do
      patchelf --set-rpath '$ORIGIN' --force-rpath "$F" 2>/dev/null || true
    done
    echo '--- bundled libs ---'
    ls dist/anime-player/lib
    echo '--- ldd shipped binary ---'
    ldd dist/anime-player/anime-player
    if ldd dist/anime-player/anime-player | grep -q 'not found'; then
      echo 'ERROR: missing shared libraries'; exit 1
    fi
    tar -czf anime-player-linux-x86_64.tar.gz -C dist anime-player
    echo "Архив: $PWD/anime-player-linux-x86_64.tar.gz"
  fi
  echo "Готово: $PWD/dist/anime-player"
else
  echo 'Пакет пропущен (--no-package). Бинарь: target/release/anime-player'
fi

echo
echo 'Сборка завершена успешно.'
