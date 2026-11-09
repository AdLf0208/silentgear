# Silent Gear — Horizon port

A port of [Silent Gear](https://github.com/SilentChaos512/Silent-Gear) by
SilentChaos512 for Horizon. This first slice is gear crafting and the tool stat
block.

- `/silentgear assemble pickaxe <head> <rod>` assembles a pickaxe from a carried
  blueprint, head and rod; the three are taken from the inventory (then the off
  hand) and consumed.
- The pickaxe's stats fold each material's properties with Silent Gear's own
  maths — weighted average, max, multiply-base, multiply-total and add, clamped —
  and those numbers drive the vanilla mining it performs.
- The material table is the jar's own data for iron, wood, stone, diamond, gold,
  flint and the empty netherite entry.

The other tools and armour, traits, grading, salvage, compound parts and recipe
types, the blueprint book and the gear-slot system are later slices. MIT —
original copyright and licence notices are kept in the port repository; see
LICENSE and LICENSE-NOTICE.md there.

**Original:** [Silent Gear](https://github.com/SilentChaos512/Silent-Gear) 4.2.1.1 by
SilentChaos512, **MIT**. The original
copyright and licence notices are kept in [LICENSE](LICENSE), and
[LICENSE-NOTICE.md](LICENSE-NOTICE.md) records where each was read from.

**Horizon:** a port for [Horizon](https://github.com/horizon-rust/horizon), Minecraft:
Java Edition 26.3 rewritten in Rust. The exact Horizon and horizon-mixins
revisions this port builds against are pinned in `compatibility.json`.

## Install

1. Take `silentgear.hnmod` from this repository (or build it below).
2. Put it in the game folder's `horizon-native-mods` folder — the game folder is the one
   holding `options.txt`.
3. Native mods are always on once installed; the first install shows a
   trust notice, and the launcher builds the mod for your game.

To try it without installing, start the game with
`--dev-mod silentgear.hnmod`.

## Build

The port is a native mod (`.hnmod`): carried content plus a Rust runtime and
source mixins.

```sh
cargo test --locked --offline                        # the pure runtime rules
python3 tools/horizon.py stage   --horizon <horizon-checkout>
python3 tools/horizon.py check   --horizon <horizon-checkout>
python3 tools/horizon.py test    --horizon <horizon-checkout>
python3 tools/horizon.py build   --horizon <horizon-checkout>   # rebuild the .hnmod
```

`tools/e2e.sh` is the in-game harness. It expects a campaign checkout
(`$CAMPAIGN_ROOT`), the shared pins under `ports/_shared/pins/`, and the
prepared 26.3 resources (`$HORIZON_RES`, default `.local/horizon-client/26.3`).

## Layout

`content/` is the shipped content; `mixins/silentgear/` holds the declarations and the runtime; `tools/` builds and verifies the port.

This is an unofficial port: Horizon is not affiliated with the original author.
The licence above is the original's and travels with every copy; see
[LICENSE](LICENSE) and [LICENSE-NOTICE.md](LICENSE-NOTICE.md).
