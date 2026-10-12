#!/bin/sh
# Drive the running app with real pointer events and read back its numbers.
#
#   probe.sh state [jq-filter]        GET /state, pretty-printed (or filtered with jq)
#   probe.sh summary                  one line per widget: id x y w h group shaded dock light tween
#   probe.sh grab ID                  the widget's nameplate point "x,y" (press here to drag it)
#   probe.sh shot OUT.png [x y w h]   screenshot a region (default: all widgets plus 40 pt)
#   probe.sh drag ID DX DY [options]  press ID's nameplate, drag by DX,DY, report mid-drag, release
#   probe.sh dragxy X1 Y1 X2 Y2 [options]   the same from and to screen points
#
# Drag options:
#   --steps N     pointer moves along the way (default 12)
#   --step-ms MS  wait between moves (default 16)
#   --hold MS     wait before reading mid-drag state (default 150, lets springs settle)
#   --shot P.png  screenshot mid-drag (widgets region); P-after.png after release
#   --alt         hold Option during the drag
#   --escape      press Escape instead of releasing (cancel); releases afterwards
#   --no-release  leave the button down (finish later with: cliclick du:X,Y)
#
# Coordinates are points on the main display, top-left origin, the same as /state.
# Needs cliclick, curl and jq. Drags and shots refuse to run while the screen is locked. PORT overrides the endpoint port (default 47800).

set -eu
PORT=${PORT:-47800}
URL="http://127.0.0.1:$PORT/state"

die() { echo "probe.sh: $*" >&2; exit 1; }
# A locked screen swallows pointer events and screenshots show only the wallpaper.
unlocked() {
  if ioreg -n Root -d1 -a 2>/dev/null | grep -A1 CGSSessionScreenIsLocked | grep -q '<true/>'; then
    die "the screen is locked: drags and screenshots can't reach the app (unlock, then retry)"
  fi
}
state() { curl -fsS --max-time 3 "$URL" || die "no /state on port $PORT (is desktop-widgets running?)"; }
# Fetch first, then filter, so a dead app fails the command (sh has no pipefail).
query() { s=$(state) || exit 1; printf '%s\n' "$s" | jq "$@"; }

summary() {
  query -r '
    (.widgets[] | [.id, (.x|round), (.y|round), (.w|round), (.h|round), "g\(.group)",
       (if .shaded then "shaded" else "open" end), (.dock // "-"), "light=\(.light)",
       "tween=\(.tween // "-")"] | @tsv),
    (if .drag then "drag: pressed=\(.drag.pressed) members=\(.drag.members|join(",")) moved=\(.drag.moved) cursor=\(.drag.cursor.x|round),\(.drag.cursor.y|round) hover_dock=\(.drag.hover_dock // "-")" else "drag: none" end)'
}

grab() {
  query -r --arg id "$1" '.widgets[] | select(.id == $id) | "\(.grab.x|round),\(.grab.y|round)"' | grep . || die "no widget '$1'"
}

region() {
  query -r '[.widgets[] | {x, y, r: (.x + .w), b: (.y + .h)}] |
    "\((map(.x)|min) - 40 | floor) \((map(.y)|min) - 40 | floor) \((map(.r)|max) - (map(.x)|min) + 80 | ceil) \((map(.b)|max) - (map(.y)|min) + 80 | ceil)"'
}

shot() {
  unlocked
  out=$1; shift
  if [ $# -ge 4 ]; then set -- "$1" "$2" "$3" "$4"; else r=$(region) || exit 1; set -- $r; fi
  [ "$1" -lt 0 ] && set -- 0 "$2" "$3" "$4"
  [ "$2" -lt 0 ] && set -- "$1" 0 "$3" "$4"
  screencapture -x -R "$1,$2,$3,$4" "$out" && echo "shot: $out ($1,$2 ${3}x$4)"
}

drag() {
  unlocked
  x1=$1; y1=$2; x2=$3; y2=$4; shift 4
  steps=12; step_ms=16; hold=150; png=""; alt=0; escape=0; release=1
  while [ $# -gt 0 ]; do
    case $1 in
      --steps) steps=$2; shift ;;
      --step-ms) step_ms=$2; shift ;;
      --hold) hold=$2; shift ;;
      --shot) png=$2; shift ;;
      --alt) alt=1 ;;
      --escape) escape=1 ;;
      --no-release) release=0 ;;
      *) die "unknown option $1" ;;
    esac
    shift
  done
  moves=""
  i=1
  while [ "$i" -le "$steps" ]; do
    moves="$moves dm:=$((x1 + (x2 - x1) * i / steps)),=$((y1 + (y2 - y1) * i / steps)) w:$step_ms"
    i=$((i + 1))
  done
  echo "drag: $x1,$y1 -> $x2,$y2 in $steps steps"
  # cliclick reads a leading - as relative, so absolute points are written =x,=y.
  # If anything fails while the button is down, let go of it (and Option) on the way out.
  [ $release = 1 ] && trap 'cliclick du:="$x2",="$y2" ku:alt 2>/dev/null' EXIT INT TERM
  [ $alt = 1 ] && cliclick kd:alt
  # shellcheck disable=SC2086
  cliclick m:="$x1",="$y1" w:50 dd:="$x1",="$y1" w:60 $moves w:"$hold"
  echo "-- mid-drag"
  summary
  [ -n "$png" ] && shot "$png"
  [ $release = 0 ] && { [ $alt = 1 ] && cliclick ku:alt; return 0; }
  [ $escape = 1 ] && { cliclick kp:esc w:50; echo "(escape)"; }
  cliclick du:="$x2",="$y2"
  trap - EXIT INT TERM
  [ $alt = 1 ] && cliclick ku:alt
  cliclick w:"$hold"
  echo "-- after release"
  summary
  [ -n "$png" ] && shot "${png%.png}-after.png"
  return 0
}

cmd=${1:-}
[ -n "$cmd" ] && shift
case $cmd in
  state) query "${1:-.}" ;;
  summary) summary ;;
  grab) [ $# -eq 1 ] || die "usage: probe.sh grab ID"; grab "$1" ;;
  shot) [ $# -ge 1 ] || die "usage: probe.sh shot OUT.png [x y w h]"; shot "$@" ;;
  drag)
    [ $# -ge 3 ] || die "usage: probe.sh drag ID DX DY [options]"
    p=$(grab "$1"); x1=${p%,*}; y1=${p#*,}; dx=$2; dy=$3; shift 3
    drag "$x1" "$y1" "$((x1 + dx))" "$((y1 + dy))" "$@" ;;
  dragxy) [ $# -ge 4 ] || die "usage: probe.sh dragxy X1 Y1 X2 Y2 [options]"; drag "$@" ;;
  *) sed -n '2,21p' "$0" | sed 's/^# \{0,1\}//'; [ -z "$cmd" ] || exit 1 ;;
esac
