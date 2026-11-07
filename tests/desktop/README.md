# desktop test driver

Runs the real app in a private headless GNOME Wayland session on the local GPU, without touching any logged-in session. `drive.py` reads window geometry, takes screenshots and sends real pointer and keyboard input through Mutter's remote desktop API. `dragsrc.py` is a GTK drag source for real file drops.

```sh
tests/desktop/asgpu.sh tests/desktop/session.sh your-script.sh
```

The first click after launch only focuses the window, so scripts start with a click on an empty area. Needs passwordless sudo and GNOME Shell.
