# JNA finds native methods, structures and callbacks by reflection, and UniFFI's bindings are
# built on it, so neither may be renamed or stripped.
-keep class com.sun.jna.** { *; }
-keep class * implements com.sun.jna.** { *; }
-keep class uniffi.** { *; }
-dontwarn java.awt.**
