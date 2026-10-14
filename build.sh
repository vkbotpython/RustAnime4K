#!/bin/bash
# Локальная сборка Anime4K Player (Linux): проверки + release + dist-пакет.
# Повторяет шаги CI (ci.yml / release.yml) локально.
#
#   ./build.sh                    # всё сразу (нужен libmpv-dev; для пакета — patchelf)
#   ./build.sh --no-video         # без libmpv (только mock-бэкенд)
#   ./build.sh --skip-checks       # пропустить fmt/clippy/test
#   ./build.sh --no-package       # только target/release без dist
#
# Пакет: системные anime4k-player-linux-x86_64.deb (apt) и
# anime4k-player-linux-x86_64.rpm (dnf) через packaging/linux/mkpackages.sh:
# /opt/anime4k-player + симлинк /usr/bin + .desktop + иконка.
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
  fi
  # Системные пакеты .deb (apt) и .rpm (dnf) вместо portable-архива.
  # Нужны dpkg-deb и rpmbuild (см. packaging/linux/mkpackages.sh).
  bash packaging/linux/mkpackages.sh
  echo "Готово: $PWD/dist/anime4k-player"
else
  echo 'Пакет пропущен (--no-package). Бинарь: target/release/anime4k-player'
fi

echo
echo 'Сборка завершена успешно.'
