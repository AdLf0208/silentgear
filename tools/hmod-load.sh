#!/usr/bin/env bash
# Proves the built content package (content/silentgear.hmod) loads on a real
# game: the shared, unmodded 1d63e0a5 game with `--dev-mod <the .hmod>` must
# register the gear items. A second launch WITHOUT `--dev-mod` is the negative
# control: the same ids must be unknown there, so the check cannot pass
# vacuously.
set -euo pipefail
export LC_ALL=C

CLONE=${CAMPAIGN_ROOT:?set CAMPAIGN_ROOT to a checkout of the ATM10-Horizon campaign tree}
PIN_BIN=$CLONE/ports/_shared/bin/horizon-1d63e0a5
CTL=${CTL:-$PIN_BIN/horizon-ctl}
GAME=${GAME:-$PIN_BIN/horizon-client}
HMOD=${HMOD:-$CLONE/ports/silentgear/content/silentgear.hmod}
RES=${RES:-${HORIZON_RES:-.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/silentgear-hmod.sock}

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -f "$HMOD" ] || { echo "missing package: $HMOD" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/silentgear-hmod-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }

cleanup() {
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "${KEEP:-0}" = 1 ]; then echo "kept scratch: $SCRATCH" >&2; else rm -rf "$SCRATCH" 2>/dev/null || true; fi
}
trap cleanup EXIT

launch() { # launch WORLD [--dev-mod PATH]
    local world=$1; shift
    rm -f "$SOCK"
    cd "$SCRATCH"
    "$CTL" --socket "$SOCK" launch \
        --game "$GAME" --world "$world" --seed 20261105 --creative --headless --wait \
        --resources "$RES" "$@" >/dev/null
    ctl wait-ready 120
    ctl wait 20
}

echo "== launch WITH the package (--dev-mod $HMOD) =="
launch SilentGearHmod --dev-mod "$HMOD"
ctl give silentgear:pickaxe 1 >/dev/null
ctl give silentgear:pickaxe_blueprint 1 >/dev/null
ctl give silentgear:pickaxe_head 1 >/dev/null
ctl give silentgear:rod 1 >/dev/null
ctl give silentgear:axe_head 1 >/dev/null
ctl wait 6
counts=$(ctl inventory | jq -c '.counts')
echo "  inventory counts: $counts"
for item in \
    silentgear:pickaxe \
    silentgear:pickaxe_blueprint \
    silentgear:pickaxe_head \
    silentgear:rod \
    silentgear:axe_head; do
    got=$(jq -r --arg item "$item" '.[$item] // 0' <<<"$counts")
    [ "$got" -ge 1 ] || fail "the package did not register $item"
done
echo "  the five gear items registered"

echo "== negative control: the same ids WITHOUT --dev-mod =="
launch SilentGearHmodNegative
ctl give silentgear:pickaxe 1 >/dev/null 2>&1 || true
ctl wait 6
counts=$(ctl inventory | jq -c '.counts')
echo "  inventory counts: $counts"
got=$(jq -r '."silentgear:pickaxe" // 0' <<<"$counts")
[ "$got" = 0 ] || fail "silentgear:pickaxe exists without the package: the check is vacuous"

echo "PASS: the package registers the five Silent Gear items, and the same launch without it registers none"
