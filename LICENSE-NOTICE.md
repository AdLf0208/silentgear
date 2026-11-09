# Licence notice — Silent Gear

This port carries the original mod's assets and data, and rewrites its behaviour
in Rust (or, for a data-tier port, declares its blocks from the same data). The
original licence and copyright notices are reproduced verbatim in
[LICENSE](LICENSE).

## Where the licence was read

- The pack jar `silent-gear-1.21.1-neoforge-4.2.1.1.jar` (kept at
  `_shared/carry-work/silentgear/jars/`) declares `license="MIT"` in
  `META-INF/neoforge.mods.toml` and carries no licence file.
- The MIT text and `Copyright (c) 2018 SilentChaos512` come from the upstream
  repository's `LICENSE` (<https://github.com/SilentChaos512/Silent-Gear>).

## The port

- Original: Silent Gear 4.2.1.1 by SilentChaos512 — MIT.
- Licence: **MIT**
- Port to Horizon by AdLf0208: the Rust runtime, the Horizon declarations and the
  build/verification tooling in this repository.
- No Mojang file is included. The carried files are the mod's own assets and
  data; `data/minecraft/tags/…` files only add this mod's ids to vanilla tags.
