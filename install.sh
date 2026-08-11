#!/bin/bash
# Instalador de Río ENARM 🐶
# Uso:  curl -fsSL https://raw.githubusercontent.com/aristeovaladezmontero-coder/cukcoo-enarm-aris/main/install.sh | bash
set -e
echo "🐶 Descargando Río ENARM..."
TMP=$(mktemp -d)
curl -fsSL -o "$TMP/app.zip" "https://github.com/aristeovaladezmontero-coder/cukcoo-enarm-aris/releases/download/latest/app-macOS.zip"
ditto -x -k "$TMP/app.zip" "$TMP"
APP=$(find "$TMP" -maxdepth 1 -name "*.app" | head -1)
NOMBRE=$(basename "$APP")
echo "📦 Instalando en /Applications..."
pkill -x menubar-app 2>/dev/null || true
rm -rf "/Applications/Cukcoo ENARM Aris.app"   # limpia la version vieja (Cukcoo)
rm -rf "/Applications/$NOMBRE"
ditto "$APP" "/Applications/$NOMBRE"
xattr -cr "/Applications/$NOMBRE"
rm -rf "$TMP"
open "/Applications/$NOMBRE"
echo "✅ ¡Listo! Busca a Río en la barra de menú (arriba a la derecha)."
