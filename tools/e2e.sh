#!/usr/bin/env bash
# In-game check for Silent Gear slice 1: the assembled pickaxe and its stats.
#
# An Overworld arena at y=100 (stone floor, clear air), creative launch, then:
#
#  A. an unassembled blueprint alone: `/silentgear assemble pickaxe iron wood`
#     is refused and no pickaxe appears;
#  B. blueprint + pickaxe head + rod: the assembly gives one
#     `silentgear:pickaxe` and consumes the three inputs; `/silentgear stats`
#     prints the iron stat block from the stack's `silentgear:gear` component;
#  C. in survival, the pickaxe breaks stone in <= 12 ticks and drops
#     cobblestone, and the held stack's damage goes 0 -> 1 (the same run's
#     durability proof);
#  D. a stick on the same block in the same 16-tick budget breaks nothing and
#     drops nothing (the differential: the drop and the speed are the gear's);
#  E. a wrong part (`axe_head` instead of a pickaxe head) is refused and adds
#     no second pickaxe.
#
# SELFTEST=1 corrupts the expected post-mining damage (2, not 1), so the run
# must exit 1; that proves the durability assertion can fail.
set -euo pipefail
export LC_ALL=C

CLONE=${CAMPAIGN_ROOT:?set CAMPAIGN_ROOT to a checkout of the ATM10-Horizon campaign tree}
PORT=$CLONE/ports/silentgear
CTL=${CTL:-${HORIZON_CTL:-$CLONE/ports/_shared/bin/horizon-1d63e0a5/horizon-ctl}}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
RES=${RES:-${HORIZON_RES:-.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/silentgear-e2e.sock}
SELFTEST=${SELFTEST:-0}
KEEP=${KEEP:-0}
WORLD=SilentGearE2E

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }
[ -f "$PORT/content/horizon-mod.toml" ] || { echo "missing content: $PORT/content" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/silentgear-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; ctl wait 6 >/dev/null; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 80 >&2 || true; exit 1; }

cleanup() {
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH" >&2
    else
        rm -rf "$SCRATCH" 2>/dev/null || true
    fi
}
trap cleanup EXIT

launch() {
    rm -f "$SOCK"
    cd "$SCRATCH"
    "$CTL" --socket "$SOCK" launch \
        --game "$GAME" --world "$WORLD" --seed 20261105 --creative --offscreen --wait \
        --resources "$RES" --dev-mod "$PORT/content" >/dev/null
    ctl wait-ready 120
    ctl wait 40
}

# A stone floor at y=100 and a clear volume above it in the Overworld.
arena() {
    ctl command "execute in minecraft:overworld run tp @s 0.5 101 -4.5"
    local loaded=false
    for _ in $(seq 1 120); do
        ctl wait 5
        if [ "$(ctl block 0 100 -8 | jq -r '.loaded // false')" = true ]; then
            loaded=true
            break
        fi
    done
    [ "$loaded" = true ] || fail "the Overworld arena chunk never loaded"
    ctl command "execute in minecraft:overworld run fill -8 100 -12 24 100 4 minecraft:stone"
    ctl command "execute in minecraft:overworld run fill -8 101 -12 24 112 4 minecraft:air"
    ctl command "execute in minecraft:overworld run kill @e[type=!player]"
    ctl command "execute in minecraft:overworld run tp @s 0.5 101 0.5"
    ctl wait 20
}

count() { ctl inventory | jq -r --arg item "$1" '.counts[$item] // 0'; }
held() { ctl inventory | jq -c '.held'; }
stone() { # stone X Y Z -> set one stone block
    ctl command "execute in minecraft:overworld run setblock $1 $2 $3 minecraft:stone" >/dev/null
    ctl wait 4
}
block_name() { ctl block "$1" "$2" "$3" | jq -r '.block // ""'; }

echo "== launch (Overworld arena, content + native bundle) =="
launch
arena
[ "$(block_name 0 100 0)" = "minecraft:stone" ] || fail "the arena floor is not stone"

echo "== A. an unassembled blueprint builds nothing =="
ctl give silentgear:pickaxe_blueprint 1 >/dev/null
ctl wait 4
[ "$(count silentgear:pickaxe_blueprint)" -ge 1 ] || fail "the blueprint item did not register/give"
say "silentgear assemble pickaxe iron wood"
refusal=$(ctl messages 10 | jq -r '.chat[]' | grep -i "carry a silentgear:pickaxe_head" | head -1 || true)
echo "  refusal: $refusal"
[ -n "$refusal" ] || fail "the assembly without parts was not refused"
[ "$(count silentgear:pickaxe)" = 0 ] || fail "an unassembled blueprint produced a pickaxe"

echo "== B. blueprint + head + rod assemble the pickaxe =="
ctl give silentgear:pickaxe_head 1 >/dev/null
ctl give silentgear:rod 1 >/dev/null
ctl wait 4
say "silentgear assemble pickaxe iron wood"
built=$(ctl messages 10 | jq -r '.chat[]' | grep -E "^Assembled silentgear:pickaxe " | head -1 || true)
echo "  $built"
[ -n "$built" ] || fail "the assembly gave no line"
[ "$(count silentgear:pickaxe)" = 1 ] || fail "the assembly did not give exactly one pickaxe"
[ "$(count silentgear:pickaxe_head)" = 0 ] || fail "the pickaxe head was not consumed"
[ "$(count silentgear:rod)" = 0 ] || fail "the rod was not consumed"
[ "$(count silentgear:pickaxe_blueprint)" = 0 ] || fail "the blueprint was not consumed"

ctl select silentgear:pickaxe >/dev/null
ctl wait 4
say "silentgear stats"
line=$(ctl messages 10 | jq -r '.chat[]' | grep -E "^durability=250/250 harvest_speed=6 enchantment_value=14 tier=iron$" | head -1 || true)
echo "  stats: $line"
[ -n "$line" ] || fail "the stat block read back wrong"
held_stack=$(held)
echo "  held: $held_stack"
jq -e '.item == "silentgear:pickaxe" and (.components // "" | contains("silentgear:gear"))' <<<"$held_stack" >/dev/null \
    || fail "the pickaxe's gear component is missing: $held_stack"

echo "== C. the pickaxe mines stone: a drop, its speed, one durability =="
ctl command "gamemode survival" >/dev/null
ctl wait 4
ctl select silentgear:pickaxe >/dev/null
stone 1 101 0
ctl break 1 101 0 --timeout 0.8 >/tmp/opencode/silentgear-break.json || true
brk=$(cat /tmp/opencode/silentgear-break.json)
echo "  break: $brk"
jq -e '.block == "minecraft:air"' <<<"$brk" >/dev/null || fail "the pickaxe did not break the stone in 16 ticks: $brk"
jq -e '.drops["minecraft:cobblestone"] == 1' <<<"$brk" >/dev/null || fail "the pickaxe broke stone without its cobblestone drop: $brk"
jq -e '.ticks <= 16' <<<"$brk" >/dev/null || fail "the pickaxe was slower than 16 ticks: $brk"
damage=$(held | jq -r '.damage // 0')
echo "  held damage after one block: $damage"
want=1
[ "$SELFTEST" = 1 ] && want=2
[ "$damage" = "$want" ] || fail "the pickaxe's damage is $damage, expected $want"

echo "== D. control: a stick in the same budget breaks nothing =="
ctl give minecraft:stick 1 >/dev/null
ctl wait 4
ctl select minecraft:stick >/dev/null
stone 1 101 2
ctl break 1 101 2 --timeout 0.8 >/tmp/opencode/silentgear-stick.json || true
stick=$(cat /tmp/opencode/silentgear-stick.json)
echo "  stick break: $stick"
[ "$(block_name 1 101 2)" = "minecraft:stone" ] || fail "a stick broke stone within 16 ticks (it cannot)"
jq -e '(.drops // {}) == {}' <<<"$stick" >/dev/null || fail "a stick produced a drop: $stick"

echo "== E. control: a wrong part is refused =="
ctl give silentgear:pickaxe_blueprint 1 >/dev/null
ctl give silentgear:axe_head 1 >/dev/null
ctl give silentgear:rod 1 >/dev/null
ctl wait 4
say "silentgear assemble pickaxe iron wood"
wrong=$(ctl messages 10 | jq -r '.chat[]' | grep -i "is not a silentgear:pickaxe_head" | head -1 || true)
echo "  refusal: $wrong"
[ -n "$wrong" ] || fail "an axe head was not refused as a pickaxe head"
[ "$(count silentgear:pickaxe)" = 1 ] || fail "the wrong-part assembly added a pickaxe"

echo "PASS: the unassembled blueprint built nothing; blueprint + head + rod gave one silentgear:pickaxe whose component reads durability=250 harvest_speed=6 enchantment_value=14 tier=iron; in survival it broke stone inside 16 ticks (8 in this run) with its cobblestone drop and took 1 damage; a stick in the same budget broke nothing and dropped nothing; an axe head was refused"
echo "EXIT:$?"
