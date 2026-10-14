#!/bin/bash
# Сборка системных пакетов .deb (apt) и .rpm (dnf) из dist/anime4k-player.
#
#   ./packaging/linux/mkpackages.sh [версия]   # default: version из Cargo.toml
#
# На выходе в корне репо:
#   anime4k-player-linux-x86_64.deb / anime4k-player-linux-x86_64.rpm
# Раскладка внутри пакета:
#   /opt/anime4k-player/ (бинарник, shaders/, lib/ с bundled libmpv, README, LICENSE)
#   /usr/bin/anime4k-player (симлинк), .desktop в /usr/share/applications,
#   иконка в /usr/share/pixmaps.
# Требует: dpkg-deb (в Debian/Ubuntu из коробки) и rpmbuild
# (sudo apt install rpm / sudo dnf install rpm-build).
# RPATH бинаря ($ORIGIN/lib) уже настроен шагом бандлинга и под /opt
# работает без изменений. Вызывать из корня репо.
set -euo pipefail
cd "$(dirname "$0")/../.."

VERSION="${1:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)}"
[ -n "$VERSION" ] || { echo 'Не удалось определить версию'; exit 1; }
echo "Версия пакетов: $VERSION"

command -v dpkg-deb >/dev/null || { echo 'Нужен dpkg-deb (в Debian/Ubuntu уже установлен)'; exit 1; }
command -v rpmbuild >/dev/null || { echo 'Нужен rpmbuild: sudo apt install rpm / sudo dnf install rpm-build'; exit 1; }

BIN=dist/anime4k-player/anime4k-player
[ -x "$BIN" ] || { echo "Нет $BIN: сначала соберите dist-пакет (build.sh / CI)"; exit 1; }
[ -f dist/anime4k-player/README.md ] || { echo 'Нет dist/anime4k-player/README.md'; exit 1; }
[ -f dist/anime4k-player/LICENSE ] || { echo 'Нет dist/anime4k-player/LICENSE'; exit 1; }
[ -d dist/anime4k-player/shaders ] || { echo 'Нет dist/anime4k-player/shaders'; exit 1; }

DESKTOP_TMPL=packaging/linux/io.github.avencores.Anime4KPlayer.desktop
ICON_SRC=assets/images/anime4k-player-icon.png
[ -f "$DESKTOP_TMPL" ] || { echo "Нет $DESKTOP_TMPL"; exit 1; }
[ -f "$ICON_SRC" ] || { echo "Нет $ICON_SRC"; exit 1; }

# --- 1. Дерево пакета ---
STAGE=pkgroot
rm -rf "$STAGE"
mkdir -p "$STAGE/opt/anime4k-player/lib" \
         "$STAGE/usr/bin" \
         "$STAGE/usr/share/applications" \
         "$STAGE/usr/share/pixmaps" \
         "$STAGE/DEBIAN"
cp "$BIN" "$STAGE/opt/anime4k-player/anime4k-player"
chmod 0755 "$STAGE/opt/anime4k-player/anime4k-player"
cp -r dist/anime4k-player/shaders "$STAGE/opt/anime4k-player/"
if [ -n "$(ls -A dist/anime4k-player/lib 2>/dev/null)" ]; then
  cp -a dist/anime4k-player/lib/. "$STAGE/opt/anime4k-player/lib/"
fi
cp dist/anime4k-player/README.md dist/anime4k-player/LICENSE "$STAGE/opt/anime4k-player/"
ln -s /opt/anime4k-player/anime4k-player "$STAGE/usr/bin/anime4k-player"
# Ярлык с абсолютными путями (в portable-шаблоне — относительные).
sed -e 's|^Exec=.*|Exec=/usr/bin/anime4k-player|' \
    -e 's|^Icon=.*|Icon=/usr/share/pixmaps/anime4k-player.png|' \
    "$DESKTOP_TMPL" > "$STAGE/usr/share/applications/io.github.avencores.Anime4KPlayer.desktop"
cp "$ICON_SRC" "$STAGE/usr/share/pixmaps/anime4k-player.png"

# --- 2. .deb ---
cat > "$STAGE/DEBIAN/control" <<EOF
Package: anime4k-player
Version: $VERSION
Section: video
Priority: optional
Architecture: amd64
Maintainer: avencores
Description: Video player with Anime4K image upscaling
 Видеоплеер с улучшением качества рисованного изображения (Anime4K, libmpv).
EOF
cat > "$STAGE/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/share/applications || true
fi
EOF
chmod 0755 "$STAGE/DEBIAN/postinst"
cp "$STAGE/DEBIAN/postinst" "$STAGE/DEBIAN/postrm"
DEB=anime4k-player-linux-x86_64.deb
rm -f "$DEB"
dpkg-deb --root-owner-group --build "$STAGE" "$DEB"
echo "--- deb info ---"
dpkg-deb --info "$DEB"
echo "--- deb contents ---"
dpkg-deb --contents "$DEB"

# --- 3. .rpm (DEBIAN внутрь rpm класть нельзя) ---
rm -rf "$STAGE/DEBIAN"
TOPDIR="$PWD/.rpmbuild"
rm -rf "$TOPDIR"
mkdir -p "$TOPDIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
cat > "$TOPDIR/SPECS/anime4k-player.spec" <<EOF
Name:           anime4k-player
Version:        $VERSION
Release:        1
Summary:        Video player with Anime4K image upscaling
License:        GPLv3+
BuildArch:      x86_64

%description
Video player with Anime4K image upscaling (libmpv).

%install
mkdir -p %{buildroot}
cp -a $PWD/$STAGE/* %{buildroot}/

%files
/opt/anime4k-player
/usr/bin/anime4k-player
/usr/share/applications/io.github.avencores.Anime4KPlayer.desktop
/usr/share/pixmaps/anime4k-player.png

%post
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/share/applications || :
fi

%postun
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/share/applications || :
fi
EOF
rpmbuild -bb --define "_topdir $TOPDIR" "$TOPDIR/SPECS/anime4k-player.spec"
RPM=anime4k-player-linux-x86_64.rpm
rm -f "$RPM"
cp "$TOPDIR/RPMS/x86_64/anime4k-player-$VERSION-1.x86_64.rpm" "$RPM"
echo '--- rpm info ---'
rpm -qpi "$RPM"
echo '--- rpm files ---'
rpm -qpl "$RPM"

rm -rf "$STAGE" "$TOPDIR"
echo "Готово: $PWD/$DEB, $PWD/$RPM"
