#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ -n "${RYUJINX_DIR:-}" ]]; then
    ryujinx_dir="$RYUJINX_DIR"
else
    if ! command -v cmd.exe >/dev/null || ! command -v wslpath >/dev/null; then
        echo "Impossible de trouver Ryujinx. Définis RYUJINX_DIR." >&2
        exit 1
    fi

    appdata="$(cmd.exe /c 'echo %APPDATA%' 2>/dev/null | tr -d '\r')"
    if [[ -z "$appdata" || "$appdata" == *'%APPDATA%'* ]]; then
        echo "Impossible de lire APPDATA. Définis RYUJINX_DIR." >&2
        exit 1
    fi
    ryujinx_dir="$(wslpath -u "$appdata")/Ryujinx"
fi

dest_dir="$ryujinx_dir/sdcard/engage/mods/Oligoludia_Plugin"
dest="$dest_dir/liboligoludia_plugin.nro"
source_nro="target/aarch64-skyline-switch/release/liboligoludia_plugin.nro"

cargo skyline build --release

if [[ ! -f "$source_nro" ]]; then
    echo "NRO introuvable : $source_nro" >&2
    exit 1
fi

mkdir -p -- "$dest_dir"
if [[ -f "$dest" && ! -e "$dest.bak" ]]; then
    cp -p -- "$dest" "$dest.bak"
fi
cp -- "$source_nro" "$dest"
echo "NRO installé : $dest"
