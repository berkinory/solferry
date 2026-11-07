#!/usr/bin/env python3
"""GTK drag source used to deliver a real text/uri-list drop into a probe window."""
import sys, gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk
uri = "file://" + sys.argv[1]
w = Gtk.Window(title="dragsrc"); w.set_default_size(220, 140); w.move(0, 0)
b = Gtk.Button(label="drag me"); w.add(b)
b.drag_source_set(Gdk.ModifierType.BUTTON1_MASK, [Gtk.TargetEntry.new("text/uri-list", 0, 0)], Gdk.DragAction.COPY)
b.connect("drag-data-get", lambda _w, _c, sel, _i, _t: sel.set_uris([uri]))
w.connect("destroy", Gtk.main_quit); w.show_all(); Gtk.main()
