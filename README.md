# Sulphur

**Sulphur** is a library for creating **instanced GZDoom launchers**.

All files saved by this library are stored under a `sulphur` directory to ensure instances are easily transferrable between implementations.

> ⚠ Due to its use of the `xdg` crate, this library currently only compiles on **UNIX systems**.

---

## Features

* Create new instances
* Track playtime
* Sort instances by playtime or last played
* Delete instances
* Add/remove Mods and IWADs, and optionally move them into the shared `sulphur` data folder
* Create and import `.brimpkg` files (ZIP archives in a trenchcoat containing instances)
* Instance-specific save folders
* Support for additional parameters per instance
* Change the command used to run GZDoom (helpful for custom paths or Flatpak installations of GZDoom)

---

## Planned Features

* Documentation
* Global additional parameters
