#!/usr/bin/env bash
# Average CPU of the given processes, plus WindowServer, over a window of
# seconds, for comparing Herdr UI against a terminal running the herdr TUI.
#
#   scripts/bench-power.sh SECONDS PID...
#
# Start Herdr UI with HERDR_FRAME_STATS=/path/to/stats.log to also record
# its redraw counters for the same window (see crates/herdr-gpui/src/frame_stats.rs).
# Package and GPU power need root, so run them alongside by hand:
#   sudo powermetrics --samplers cpu_power,gpu_power -i 1000 -n SECONDS
set -euo pipefail
[[ $(uname -s) == Darwin ]] || { echo 'macOS required' >&2; exit 1; }
[[ $# -ge 2 && $1 =~ ^[1-9][0-9]*$ ]] || { echo 'Usage: bench-power.sh SECONDS PID...' >&2; exit 2; }
seconds=$1
shift
server=$(pgrep -x WindowServer | head -n 1)
args=()
for pid in "$@" "$server"; do
    # `kill -0` fails with EPERM on WindowServer, which root owns.
    [[ $pid =~ ^[0-9]+$ ]] && ps -p "$pid" >/dev/null || { echo "No such process: $pid" >&2; exit 1; }
    args+=(-pid "$pid")
done
# The first sample has no previous one to measure against, so take one extra
# and drop it.
top -l $((seconds + 1)) -s 1 -stats pid,command,cpu "${args[@]}" | awk '
    /^PID/ { sample++; next }
    sample > 1 && /^[0-9]+ / {
        sum[$1] += $NF; n[$1]++
        name[$1] = $2
    }
    END {
        for (pid in sum) printf "%-8s %-20s %6.1f%%  (%d samples)\n", pid, name[pid], sum[pid] / n[pid], n[pid]
    }' | sort -k3 -rn
