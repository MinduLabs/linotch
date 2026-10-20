#!/usr/bin/env bash
# Builds inside Docker, so the host needs nothing but Docker. Output: ./dist/
#   ./build.sh appimage   only the AppImage
#   ./build.sh deb        only the .deb
#   ./build.sh            both
set -euo pipefail
cd "$(dirname "$0")"

case "${1:-all}" in
    appimage) BUNDLES=appimage ;;
    deb)      BUNDLES=deb ;;
    all)      BUNDLES=appimage,deb ;;
    *) echo "usage: $0 [appimage|deb]" >&2; exit 1 ;;
esac

IMAGE=linotch-builder
docker build -t "$IMAGE" docker

mkdir -p dist
docker run --rm \
    -v "$PWD":/src:ro \
    -v "$PWD/dist":/out \
    -v linotch-cargo:/opt/cargo/registry \
    -v linotch-target:/work/src-tauri/target \
    -v linotch-pnpm:/root/.local/share/pnpm \
    -v linotch-tauri-cache:/root/.cache/tauri \
    -e BUNDLES="$BUNDLES" -e HOST_UID="$(id -u)" -e HOST_GID="$(id -g)" \
    "$IMAGE" bash -euo pipefail -c '
        # Work on a copy so the container never touches the host node_modules / target.
        rsync -a --delete --exclude node_modules --exclude src-tauri/target --exclude .svelte-kit \
              --exclude build --exclude dist --exclude .git /src/ /work/
        pnpm install --frozen-lockfile
        pnpm tauri build --bundles "$BUNDLES"
        B=src-tauri/target/release/bundle
        for b in ${BUNDLES//,/ }; do
            rm -rf "/out/$b"
            mkdir -p "/out/$b"
        done
        if [[ $BUNDLES == *appimage* ]]; then
            # Tauri packs with zstd at default level and 128K blocks. The AppImage runtime only reads gzip/zstd,
            # so repack the same AppDir with zstd -22 and 1M blocks: 77 MB -> 70 MB. AppImage = runtime + squashfs.
            img=$(ls $B/appimage/*.AppImage)
            offset=$("$img" --appimage-offset)
            head -c "$offset" "$img" > /tmp/runtime
            mksquashfs $B/appimage/*.AppDir /tmp/fs.sqfs -comp zstd -Xcompression-level 22 -b 1M -root-owned -noappend -quiet
            cat /tmp/runtime /tmp/fs.sqfs > "/out/appimage/$(basename "$img")"
            chmod +x /out/appimage/*.AppImage
        fi
        [[ $BUNDLES == *deb* ]] && cp $B/deb/*.deb /out/deb/
        chown -R "$HOST_UID:$HOST_GID" /out
    '

echo
echo "Done:"
find dist -type f \( -name '*.AppImage' -o -name '*.deb' \)
