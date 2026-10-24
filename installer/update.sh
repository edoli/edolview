#!/bin/sh
# A separate process owns the downloaded package after the UI commits and exits.
set -eu
ROOT=$EDOLVIEW_UPDATE_DIR
EXE=$EDOLVIEW_UPDATE_EXE
PACKAGE=$EDOLVIEW_UPDATE_PACKAGE
PARENT=$EDOLVIEW_UPDATE_PARENT
RESULT=$EDOLVIEW_CONFIG_DIR/update-result.txt
committed=0
replaced=0
mount_dir=
stage=
backup=
asset_stage=
asset_backup=
asset_target=
case "${ROOT##*/}" in edolview-update-*) ;; *) exit 1 ;; esac
[ "${PACKAGE%/*}" = "$ROOT" ] || exit 1

verify() {
    if command -v sha256sum >/dev/null 2>&1; then
        actual=$(sha256sum "$PACKAGE" | cut -d ' ' -f 1)
    else
        actual=$(shasum -a 256 "$PACKAGE" | cut -d ' ' -f 1)
    fi
    [ "$actual" = "$EDOLVIEW_UPDATE_HASH" ]
}

finish() {
    status=$?
    trap - EXIT
    if [ "$status" -ne 0 ]; then
        printf 'Update failed. %s\n' "$(tail -c 2048 "$ROOT/install.log" 2>/dev/null || true)" > "$RESULT"
        if [ "$replaced" -eq 1 ] && [ -n "$backup" ] && [ -e "$backup" ]; then
            mv "$backup" "$EXE" || true
        fi
        if [ -n "$asset_backup" ] && [ -d "$asset_backup" ]; then
            if [ -d "$asset_target" ]; then mv "$asset_target" "$ROOT/failed-assets"; fi
            mv "$asset_backup" "$asset_target" || true
        fi
    else
        rm -f "$RESULT"
    fi
    if [ -n "$mount_dir" ]; then hdiutil detach "$mount_dir" -quiet || true; fi
    if [ "$committed" -eq 1 ]; then
        nohup sh "$ROOT/restart.sh" >/dev/null 2>&1 &
        # Leave restart.sh until its child has opened it.
        sleep 1
    fi
    if [ -n "$stage" ] && [ -f "$stage" ]; then rm -f "$stage"; fi
    if [ -n "$backup" ] && [ -f "$backup" ]; then rm -f "$backup"; fi
    # All extraction directories are freshly created children of this update's root.
    for owned in "$ROOT/payload" "$ROOT/failed-assets"; do
        case "$owned" in "$ROOT"/*) rm -rf "$owned" ;; esac
    done
    for owned in "$asset_stage" "$asset_backup"; do
        case "$owned" in "${EXE%/*}/".edolview-assets.*) rm -rf "$owned" ;; esac
    done
    rm -f "$ROOT/context.json" "$ROOT/update.sh" "$ROOT/install-mac.sh" "$ROOT/restart.sh" "$ROOT/ready" "$ROOT/go" "$ROOT/go.pending" "$PACKAGE"
    if [ -n "$mount_dir" ]; then rmdir "$mount_dir" 2>/dev/null || true; fi
    if [ "$status" -eq 0 ]; then rm -f "$ROOT/install.log"; fi
    rmdir "$ROOT" 2>/dev/null || true
}
trap finish EXIT
exec 2>"$ROOT/install.log"
verify
case "${PACKAGE##*/}" in
    edolview-linux.zip) command -v unzip >/dev/null 2>&1 ;;
esac
printf ready > "$ROOT/ready"
while [ ! -f "$ROOT/go" ]; do
    if ! kill -0 "$PARENT" 2>/dev/null; then
        [ -f "$ROOT/go" ] || exit 0
    fi
    sleep 0.05
done
committed=1
while kill -0 "$PARENT" 2>/dev/null; do sleep 0.1; done
verify
case "${PACKAGE##*/}" in
    edolview-linux.zip|edolview-mac.zip)
        stage=$(mktemp "${EXE}.update.XXXXXX")
        if [ "${PACKAGE##*/}" = 'edolview-mac.zip' ]; then
            unzip -p "$PACKAGE" Edolview.app/Contents/MacOS/edolview > "$stage"
        else
            unzip -p "$PACKAGE" edolview > "$stage"
        fi
        [ -s "$stage" ]
        chmod 755 "$stage"
        backup=$(mktemp "${EXE}.old.XXXXXX")
        cp -p "$EXE" "$backup"
        asset_target=${EXE%/*}/colormap
        asset_stage=$(mktemp -d "${EXE%/*}/.edolview-assets.XXXXXX")
        # The new archive has only an executable and relative colormap files.
        unzip -Z1 "$PACKAGE" | while IFS= read -r entry; do
            case "$entry" in
                edolview|colormap/mono/*.glsl|colormap/rgb/*.glsl) ;;
                *) echo 'Unexpected portable archive entry' >&2; exit 1 ;;
            esac
            case "$entry" in *../*|/*|*\\*) exit 1 ;; esac
        done
        unzip -q "$PACKAGE" 'colormap/*' -d "$asset_stage"
        if [ -d "$asset_target" ]; then
            asset_backup=$(mktemp -d "${EXE%/*}/.edolview-assets.XXXXXX")
            rmdir "$asset_backup"
            mv "$asset_target" "$asset_backup"
        fi
        replaced=1
        mv "$asset_stage/colormap" "$asset_target"
        mv -f "$stage" "$EXE"
        stage=
        ;;
    edolview-installer-appimage.AppImage)
        backup=$(mktemp "${EXE}.old.XXXXXX")
        cp -p "$EXE" "$backup"
        stage=$(mktemp "${EXE}.update.XXXXXX")
        cp "$PACKAGE" "$stage"
        chmod 755 "$stage"
        replaced=1
        mv -f "$stage" "$EXE"
        stage=
        ;;
    edolview-installer-deb.deb)
        [ "$(dpkg-deb -f "$PACKAGE" Version)" = "$EDOLVIEW_UPDATE_VERSION" ]
        pkexec /usr/bin/dpkg -i "$PACKAGE"
        ;;
    edolview-installer-dmg.dmg)
        case "$EXE" in */Contents/MacOS/edolview) ;; *) echo 'Not an app bundle' >&2; exit 1 ;; esac
        bundle=${EXE%/Contents/MacOS/edolview}
        mount_dir=$ROOT/mount
        mkdir "$mount_dir"
        hdiutil attach -nobrowse -quiet -mountpoint "$mount_dir" "$PACKAGE"
        codesign --verify --deep --strict "$mount_dir/Edolview.app"
        [ "$(defaults read "$mount_dir/Edolview.app/Contents/Info" CFBundleShortVersionString)" = "$EDOLVIEW_UPDATE_VERSION" ]
        # ditto handles app-bundle metadata. Authorization, when needed, is
        # supplied by macOS; no password is read or stored by the application.
        cat > "$ROOT/install-mac.sh" <<'MACINSTALL'
#!/bin/sh
set -eu
source_app=$1
target_app=$2
parent=${target_app%/*}
if [ ! -w "$parent" ] || [ ! -w "$target_app" ]; then exit 77; fi
work=$(mktemp -d "$parent/.edolview-update.XXXXXX")
cleanup() {
    if [ -d "$work/previous.app" ] && [ ! -d "$target_app" ]; then mv "$work/previous.app" "$target_app"; fi
    case "$work" in "$parent/".edolview-update.*) rm -rf "$work" ;; esac
}
trap cleanup EXIT
ditto "$source_app" "$work/new.app"
codesign --verify --deep --strict "$work/new.app"
mv "$target_app" "$work/previous.app"
mv "$work/new.app" "$target_app"
MACINSTALL
        osascript - "$ROOT/install-mac.sh" "$mount_dir/Edolview.app" "$bundle" <<'APPLESCRIPT'
on run argv
    set installScript to item 1 of argv
    set sourceApp to item 2 of argv
    set targetApp to item 3 of argv
    set operation to "/bin/sh " & quoted form of installScript & " " & quoted form of sourceApp & " " & quoted form of targetApp
    try
        do shell script operation
    on error errorMessage number errorNumber
        if errorNumber is 77 then
            do shell script operation with administrator privileges
        else
            error errorMessage number errorNumber
        end if
    end try
end run
APPLESCRIPT
        ;;
    *) echo 'Unsupported update package' >&2; exit 1 ;;
esac
