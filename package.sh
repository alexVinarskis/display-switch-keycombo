#!/bin/bash

# exit when any command fails
set -e

PACKAGE="display-switch-keycombo"
NAME="display-switch-keycombo"

PATH_EXEC="/usr/local/bin/display_switch_keycombo"
APP_DIR="./package$PATH_EXEC"
DEB_DIR="./package/DEBIAN"
SERVICE_NAME="display-switch-keycombo.service"
CONFIG_NAME="display-switch-keycombo.ini"
TAG=$(date '+%Y%m%d-%H%M%S')-$(git rev-parse --short HEAD)
VERSION="0.9.0-${TAG}"

rm -rf ./package
mkdir -p ./package/usr/local/bin
mkdir -p ./package/etc/systemd/system/
mkdir -p ./package/root/.config/display-switch-keycombo
mkdir -p "$DEB_DIR"

# Compile release app
cargo build --release

# Copy application's files
cp target/release/display_switch_keycombo   "$APP_DIR"
cp ./resources/"${SERVICE_NAME}"            ./package/etc/systemd/system/
cp ./resources/"${CONFIG_NAME}"             ./package/root/.config/display-switch-keycombo/

ARCHITECTURE="amd64"
PRIORITY="standard"
MAINTAINER="alexVinarskis <alex.vinarskis@gmail.com>"
HOMEPAGE="https://github.com/alexVinarskis/display-switch-keycombo"
DEPENDS="libxi-dev, xorg-dev"
DESCRIPTION="Crossplatform KVM switch/multi-input monitors control"

# Create control file of .deb
touch "$DEB_DIR"/control
echo "Package: ${PACKAGE}"                  >> "$DEB_DIR"/control
echo "Version: ${VERSION}"                  >> "$DEB_DIR"/control
echo "Architecture: ${ARCHITECTURE}"        >> "$DEB_DIR"/control
echo "Maintainer: ${MAINTAINER}"            >> "$DEB_DIR"/control
echo "Priority: ${PRIORITY}"                >> "$DEB_DIR"/control
echo "Description: ${DESCRIPTION}"          >> "$DEB_DIR"/control
echo "Depends: ${DEPENDS}"                  >> "$DEB_DIR"/control
echo "Homepage: ${HOMEPAGE}"                >> "$DEB_DIR"/control

# Create postinstall file of .deb
touch "$DEB_DIR"/postinst && chmod 755 "$DEB_DIR"/postinst
echo "sudo groupadd i2c || true" >> "$DEB_DIR"/postinst
echo "echo 'KERNEL==\"i2c-[0-9]*\", GROUP=\"i2c\"' | sudo tee -a /etc/udev/rules.d/10-local_i2c_group.rules || true" >> "$DEB_DIR"/postinst
echo "sudo sudo udevadm control --reload-rules || true && sudo udevadm trigger || true" >> "$DEB_DIR"/postinst
echo "sudo usermod -aG i2c $(whoami) || true" >> "$DEB_DIR"/postinst
echo "sudo systemctl daemon-reload" >> "$DEB_DIR"/postinst
echo "sudo systemctl enable ${SERVICE_NAME}" >> "$DEB_DIR"/postinst
echo "sudo systemctl start ${SERVICE_NAME}" >> "$DEB_DIR"/postinst

# Create preremove file of .deb
touch "$DEB_DIR"/prerm && chmod 755 "$DEB_DIR"/prerm
echo "sudo systemctl disable ${SERVICE_NAME}" >>  ./package/DEBIAN/prerm
echo "sudo systemctl stop ${SERVICE_NAME}" >>  ./package/DEBIAN/prerm
echo "sudo systemctl daemon-reload" >> ./package/DEBIAN/prerm

# Package
dpkg-deb --build --root-owner-group ./package
mv ./package.deb ./${PACKAGE}_${VERSION}_${ARCHITECTURE}.deb
