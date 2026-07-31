#!/bin/zsh
# Measure Bloom's REAL memory: the main process plus the three WebKit XPC services
# it drives. Every earlier measurement grepped for "bloom" in the process name and
# silently missed all three, which is how the site ended up claiming 39 MB for
# something Activity Monitor shows at ~174 MB.
#
#   ./measure-memory.sh baseline          # run BEFORE starting Bloom
#   ./measure-memory.sh sample "garden"   # run while Bloom is up, desktop VISIBLE
#
# Reports phys_footprint — the same metric Activity Monitor's "Memory" column shows,
# and the same one the Wallpaper Engine / Lively figures we compare against use.
# RSS is deliberately not used: it excludes compressed memory and reads far too low.

set -e
BASE=/tmp/bloom-webkit-baseline.txt

webkit_pids () { ps -axo pid,command | grep "WebKit.framework" | grep -v grep | awk '{print $1}'; }

footprint_mb () {  # $1 = pid -> current phys_footprint in MB
  footprint -p "$1" 2>/dev/null \
    | grep -iE "phys_footprint:" | tail -1 \
    | grep -oE '[0-9.]+ ?[KMG]B' | tail -1 \
    | awk '{v=$1; u=$2; if(u=="KB"||$0~/KB/) v=v/1024; else if(u=="GB"||$0~/GB/) v=v*1024; printf "%.1f", v}'
}

# INSTANTANEOUS cpu across the given pids.
# Do NOT use `ps -o %cpu` here: on macOS that is the average over the process's whole
# lifetime, so a process that was busy ten minutes ago still reports non-zero while
# completely idle. It made an earlier version of this script report AWAKE for a
# genuinely suspended app, which is exactly the kind of error this script exists to
# catch. `top -l 2` discards its first (lifetime-average) sample; the second is real.
cpu_now () {
  local args=""
  for p in $@; do args="$args -pid $p"; done
  top -l 2 ${=args} -stats cpu 2>/dev/null \
    | tail -n +2 | grep -E '^[0-9.]+$' | tail -n $# \
    | awk '{s+=$1} END {printf "%.1f", s+0}'
}

case "$1" in
  baseline)
    webkit_pids > $BASE
    echo "baselined $(wc -l < $BASE | tr -d ' ') pre-existing WebKit processes"
    echo "now launch Bloom, make the desktop visible, then: $0 sample <label>"
    ;;

  sample)
    LABEL="${2:-unlabelled}"
    BLOOM=$(pgrep -x bloom | head -1)
    [ -z "$BLOOM" ] && { echo "bloom is not running"; exit 1; }

    # Bloom's WebKit processes = whatever exists now minus what existed before.
    if [ -f $BASE ]; then
      KIDS=$(webkit_pids | grep -vxF -f $BASE || true)
    else
      echo "!! no baseline file; assuming ALL WebKit processes are Bloom's (may overcount)"
      KIDS=$(webkit_pids)
    fi

    PIDS="$BLOOM $(echo $KIDS | tr '\n' ' ')"
    echo "=== $LABEL ==="
    echo "pids: $PIDS"
    echo ""

    # Three samples 10s apart so a single unlucky moment can't define the number.
    for round in 1 2 3; do
      TOTAL=0
      for p in ${=PIDS}; do
        ps -p $p > /dev/null 2>&1 || continue
        MB=$(footprint_mb $p); [ -z "$MB" ] && MB=0
        TOTAL=$(echo "$TOTAL + $MB" | bc)
      done
      CPU=$(cpu_now ${=PIDS})
      STATE=$([ "$(echo "$CPU > 0.5" | bc)" = "1" ] && echo "rendering" || echo "idle/suspended")
      printf "  sample %d: %8.1f MB   cpu %5s%%   [%s]\n" $round $TOTAL "$CPU" "$STATE"
      [ $round -lt 3 ] && sleep 10
    done

    echo ""
    echo "per-process breakdown (final sample):"
    for p in ${=PIDS}; do
      ps -p $p > /dev/null 2>&1 || continue
      NAME=$(ps -o comm= -p $p | sed 's|.*/||')
      printf "  %8.1f MB  %s\n" "$(footprint_mb $p)" "$NAME"
    done
    ;;

  *) echo "usage: $0 baseline | sample <label>"; exit 1 ;;
esac
