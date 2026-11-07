#!/usr/bin/env python3
"""Drives the private GNOME session: window geometry, screenshots and real pointer drags."""
import json, os, sys, time, gi
HOVER = float(os.environ.get("HOVER", ".3"))
gi.require_version("Gio", "2.0")
from gi.repository import Gio, GLib
bus = Gio.bus_get_sync(Gio.BusType.SESSION)
def call(name, path, iface, method, args=None, rt=None):
    return bus.call_sync(name, path, iface, method, args, rt, Gio.DBusCallFlags.NONE, 15000).unpack()
def windows():
    js = "JSON.stringify(global.get_window_actors().map(a=>{const w=a.meta_window;const r=w.get_frame_rect();return {t:w.get_title(),c:w.get_wm_class(),x:r.x,y:r.y,w:r.width,h:r.height}}))"
    ok, out = call("org.gnome.Shell", "/org/gnome/Shell", "org.gnome.Shell", "Eval", GLib.Variant("(s)", (js,)))
    return json.loads(json.loads(out))
def shot(path):
    return call("org.gnome.Shell", "/org/gnome/Shell/Screenshot", "org.gnome.Shell.Screenshot", "Screenshot", GLib.Variant("(bbs)", (False, False, path)))
def drag(src, dst):
    sp = call("org.gnome.Mutter.RemoteDesktop", "/org/gnome/Mutter/RemoteDesktop", "org.gnome.Mutter.RemoteDesktop", "CreateSession")[0]
    I = "org.gnome.Mutter.RemoteDesktop.Session"; N = "org.gnome.Mutter.RemoteDesktop"
    call(N, sp, I, "Start")
    rel = lambda dx, dy: call(N, sp, I, "NotifyPointerMotionRelative", GLib.Variant("(dd)", (dx, dy)))
    btn = lambda down: call(N, sp, I, "NotifyPointerButton", GLib.Variant("(ib)", (272, down)))
    def pos():
        ok, out = call("org.gnome.Shell", "/org/gnome/Shell", "org.gnome.Shell", "Eval", GLib.Variant("(s)", ("JSON.stringify(global.get_pointer())",)))
        return json.loads(json.loads(out))[:2]
    def goto(t, tries=40):
        for _ in range(tries):
            x, y = pos(); dx, dy = t[0] - x, t[1] - y
            if abs(dx) < 3 and abs(dy) < 3:
                return True
            rel(dx * .7, dy * .7); time.sleep(.03)
        return False
    print("at-src", goto(src), pos()); time.sleep(.4); btn(True); time.sleep(.3)
    mid = ((src[0] + dst[0]) // 2, (src[1] + dst[1]) // 2)
    print("mid", goto(mid), pos()); time.sleep(.2)
    print("at-dst", goto(dst), pos()); time.sleep(.8); btn(False); time.sleep(.8)
    call(N, sp, I, "Stop")
def session():
    path = call("org.gnome.Mutter.RemoteDesktop", "/org/gnome/Mutter/RemoteDesktop", "org.gnome.Mutter.RemoteDesktop", "CreateSession")[0]
    call("org.gnome.Mutter.RemoteDesktop", path, "org.gnome.Mutter.RemoteDesktop.Session", "Start")
    return path

def pointer():
    ok, out = call("org.gnome.Shell", "/org/gnome/Shell", "org.gnome.Shell", "Eval", GLib.Variant("(s)", ("JSON.stringify(global.get_pointer())",)))
    return json.loads(json.loads(out))[:2]

def click(x, y):
    hide_overview()
    N, I = "org.gnome.Mutter.RemoteDesktop", "org.gnome.Mutter.RemoteDesktop.Session"
    path = session()
    for _ in range(50):
        px, py = pointer()
        if abs(x - px) < 2 and abs(y - py) < 2:
            break
        call(N, path, I, "NotifyPointerMotionRelative", GLib.Variant("(dd)", ((x - px) * .7, (y - py) * .7)))
        time.sleep(.02)
    for dx in (4, -4, 4, -4):
        call(N, path, I, "NotifyPointerMotionRelative", GLib.Variant("(dd)", (dx, 0)))
        time.sleep(.08)
    time.sleep(HOVER)
    call(N, path, I, "NotifyPointerButton", GLib.Variant("(ib)", (272, True)))
    time.sleep(.08)
    call(N, path, I, "NotifyPointerButton", GLib.Variant("(ib)", (272, False)))
    time.sleep(.2)
    call(N, path, I, "Stop")

def type_text(text):
    N, I = "org.gnome.Mutter.RemoteDesktop", "org.gnome.Mutter.RemoteDesktop.Session"
    path = session()
    time.sleep(.8)
    for char in text:
        for down in (True, False):
            call(N, path, I, "NotifyKeyboardKeysym", GLib.Variant("(ub)", (ord(char), down)))
        time.sleep(.02)
    time.sleep(.2)
    call(N, path, I, "Stop")

def shell_eval(js):
    return call("org.gnome.Shell", "/org/gnome/Shell", "org.gnome.Shell", "Eval", GLib.Variant("(s)", (js,)))[1]

def hide_overview():
    for _ in range(40):
        shell_eval("Main.overview.hide()")
        time.sleep(.25)
        if shell_eval("Main.overview.visible") == "false" and shell_eval("Main.overview.animationInProgress") == "false":
            return
    print("overview stayed open")

if __name__ == "__main__":
    cmd = sys.argv[1]
    if cmd == "windows": print(json.dumps(windows(), ensure_ascii=False))
    elif cmd == "shot": print(shot(sys.argv[2]))
    elif cmd == "click": click(int(sys.argv[2]), int(sys.argv[3]))
    elif cmd == "type": type_text(sys.argv[2])
    elif cmd == "ready": hide_overview()
    elif cmd == "drag":
        js = "Main.overview.hide(); global.get_window_actors().forEach(a=>{const w=a.meta_window; const src=(w.get_title()||'').includes('dragsrc'); w.move_frame(true, src?10:700, src?50:100);}); 'ok'"
        call("org.gnome.Shell", "/org/gnome/Shell", "org.gnome.Shell", "Eval", GLib.Variant("(s)", (js,))); time.sleep(1.5)
        ws = windows(); s = next(w for w in ws if "dragsrc" in (w["t"] or ""))
        d = next(w for w in ws if w is not s)
        print("src", s, "dst", d)
        offset = sys.argv[2:4]
        target = (d["x"] + int(offset[0]), d["y"] + int(offset[1])) if offset else (d["x"] + d["w"] // 2, d["y"] + d["h"] // 2)
        drag((s["x"] + s["w"] // 2, s["y"] + s["h"] // 2), target)
