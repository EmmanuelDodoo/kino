#!/bin/bash

set -euo pipefail

# GST_ROOT="${GST_ROOT:-C:/Program Files/Git/gstreamer/1.0/msvc_x86_64}"
# DIST_DIR="${DIST_DIR:-dist/windows}"
GST_ROOT="${GST_ROOT:-C:/Program Files/gstreamer/1.0/msvc_x86_64}"
DIST_DIR="dist/windows"

BIN_DIR="$GST_ROOT/bin"

DLLS=(
    "avcodec-61.dll"
    "avfilter-10.dll"
    "avformat-61.dll"
    "avutil-59.dll"
    "bz2.dll"
    "dav1d.dll"
    "ffi-7.dll"
    "FLAC-8.dll"
    "gio-2.0-0.dll"
    "glib-2.0-0.dll"
    "gmodule-2.0-0.dll"
    "gobject-2.0-0.dll"
    "gstapp-1.0-0.dll"
    "gstaudio-1.0-0.dll"
    "gstbase-1.0-0.dll"
    "gstcodecparsers-1.0-0.dll"
    "gstpbutils-1.0-0.dll"
    "gstreamer-1.0-0.dll"
    "gstriff-1.0-0.dll"
    "gstrtp-1.0-0.dll"
    "gsttag-1.0-0.dll"
    "gstvideo-1.0-0.dll"
    "intl-8.dll"
    "ogg-0.dll"
    "openh264-7.dll"
    "opus-0.dll"
    "orc-0.4-0.dll"
    "pcre2-8-0.dll"
    "swresample-5.dll"
    "swscale-8.dll"
    "vorbis-0.dll"
    "vorbisenc-2.dll"
    "z-1.dll"
)

LIBS=(
    "gstapp.dll"
    "gstaudioconvert.dll"
    "gstaudioparsers.dll"
    "gstaudioresample.dll"
    "gstautodetect.dll"
    "gstcoreelements.dll"
    "gstdav1d.dll"
    "gstdeinterlace.dll"
    "gstflac.dll"
    "gstisomp4.dll"
    "gstlibav.dll"
    "gstmatroska.dll"
    "gstopenh264.dll"
    "gstopus.dll"
    "gstplayback.dll"
    "gstsoundtouch.dll"
    "gstsubparse.dll"
    "gsttypefindfunctions.dll"
    "gstvideoconvertscale.dll"
    "gstvideofilter.dll"
    "gstvideoparsersbad.dll"
    "gstvolume.dll"
    "gstvorbis.dll"
    "gstwasapi.dll"
    "gstwasapi2.dll"
)

mkdir -p "$DIST_DIR"

cp target/release/kino.exe "$DIST_DIR"

echo "Using GStreamer: $GST_ROOT"

for dll in "${DLLS[@]}"; do
    src="$BIN_DIR/$dll"
    dst="$DIST_DIR/$dll"

    if [[ ! -f "$src" ]]; then
        echo "error: missing $src"
        exit 1
    fi

    echo "Copying $dll"
    cp "$src" "$dst"
done

echo
echo "Copied runtime DLL(s)."

echo
echo "Copying lib"

mkdir -p "$DIST_DIR/lib/gstreamer-1.0"

for dll in "${LIBS[@]}"; do
    src="$GST_ROOT/lib/gstreamer-1.0/$dll"
    dst="$DIST_DIR/lib/gstreamer-1.0/$dll"

    if [[ ! -f "$src" ]]; then
        echo "error: missing $src"
        exit 1
    fi

    echo "Copying $dll"
    cp "$src" "$dst"
done

cp -r "$GST_ROOT/lib/gio" "$DIST_DIR/lib"
cp -r "$GST_ROOT/lib/glib-2.0" "$DIST_DIR/lib"

echo
echo "Copied library"

echo
echo "Copying extras"
cp -r "$GST_ROOT/libexec" "$DIST_DIR"

SHARE=(
    "glib-2.0"
    "gobject-introspection-1.0"
    "gstreamer"
    "gstreamer-1.0"
    "licenses"
)

mkdir -p "$DIST_DIR/share"

for shr in "${SHARE[@]}"; do
    src="$GST_ROOT/share/$shr"
    dst="$DIST_DIR/share/$shr"

    if [[ ! -d "$src" ]]; then
        echo "error: missing $src"
        exit 1
    fi

    echo "Copying $shr"
    cp -r "$src" "$dst"
done

echo
echo "Done!"
