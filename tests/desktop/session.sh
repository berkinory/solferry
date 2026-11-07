#!/bin/bash
# Starts a private headless GNOME Wayland compositor on the Intel GPU, then runs the command given.
# Everything lives inside dbus-run-session, so the user's real session bus is untouched.
export XDG_RUNTIME_DIR=${BENCH_RUNTIME:-/tmp/beam-desktop-run}; mkdir -p $XDG_RUNTIME_DIR; chmod 700 $XDG_RUNTIME_DIR
export GSETTINGS_BACKEND=keyfile XDG_CONFIG_HOME=$XDG_RUNTIME_DIR/config XDG_SESSION_TYPE=wayland
[ -n "$BENCH_SOFT" ] && export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json LIBGL_ALWAYS_SOFTWARE=1
mkdir -p $XDG_CONFIG_HOME/glib-2.0/settings
cat > $XDG_CONFIG_HOME/glib-2.0/settings/keyfile <<KF
[org/gnome/shell]
disable-user-extensions=false
enabled-extensions=['ubuntu-appindicators@ubuntu.com']

[org/gnome/mutter]
dynamic-workspaces=false

[org/gnome/desktop/interface]
color-scheme='${BEAM_SCHEME:-default}'

[org/gnome/desktop/wm/preferences]
num-workspaces=1
KF
exec dbus-run-session -- bash -c '
gnome-shell --unsafe-mode --headless --wayland --wayland-display=beam-0 --virtual-monitor 1280x800 >$XDG_RUNTIME_DIR/shell.log 2>&1 &
S=$!
for i in $(seq 1 60); do [ -S $XDG_RUNTIME_DIR/beam-0 ] && break; sleep .25; done
sleep 4
export WAYLAND_DISPLAY=beam-0
"$@"
RC=$?
kill $S 2>/dev/null; wait $S 2>/dev/null
exit $RC
' _ "$@"
