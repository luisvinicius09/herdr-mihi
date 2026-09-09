#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════════════════
# THROWAWAY PROTOTYPE — delete once ticket 010's brief is agreed.
# herdr-mihi "picker" pane mock (wayfinder ticket 010). Renders the UI + flow
# to react to. It PRINTS the install commands it *would* run — it never runs
# anything and needs no herdr. Run:  bash .wayfinder/prototypes/010-picker-mock.sh
# ══════════════════════════════════════════════════════════════════════════
set -uo pipefail

OWNER="luisvinicius09/herdr-mihi"
names=(auto-title lazygit navigator radar space-colors)
state=(installed available available available available)
vers=(1.2.0 0.4.0 0.2.1 0.1.0 0.3.0)
desc=("tab titles follow the work in each tab"
      "git popup running your OWN lazygit"
      "fuzzy-jump to any workspace / worktree"
      "at-a-glance agent status overview"
      "a distinct color per workspace")
sel=(0 0 0 0 0)
n=${#names[@]}; cur=0
E=$'\e'
cls(){ printf "%s[2J%s[H" "$E" "$E"; }
rev(){ printf "%s[7m%s%s[0m" "$E" "$1" "$E"; }
dim(){ printf "%s[2m%s%s[0m\n" "$E" "$1" "$E"; }
count_sel(){ local c=0 s; for s in "${sel[@]}"; do ((c+=s)); done; echo "$c"; }

browse(){
  cls
  echo "  herdr-mihi · pick your plugins"
  dim "  up/down or j/k move · space select · a all · enter install · q quit"
  echo "  ───────────────────────────────────────────────────────────────"
  local i row box mark
  for ((i=0;i<n;i++)); do
    box="[ ]"; [[ ${sel[i]} -eq 1 ]] && box="[x]"
    if [[ ${state[i]} == installed ]]; then mark="* installed"; else mark="o available"; fi
    row=$(printf "%s  %-13s %-13s v%-6s %s" "$box" "${names[i]}" "$mark" "${vers[i]}" "${desc[i]}")
    if [[ $i -eq $cur ]]; then printf "  > "; rev "$row"; echo; else echo "    $row"; fi
  done
  echo "  ───────────────────────────────────────────────────────────────"
  dim "  $(count_sel) selected · network: none · installs ONLY from $OWNER"
}

confirm(){
  cls
  echo "  Confirm — the ONLY commands that will run:"; echo
  local i any=0
  for ((i=0;i<n;i++)); do
    [[ ${sel[i]} -eq 1 ]] || continue; any=1
    printf "    herdr plugin install %s --ref %s-latest\n" "$OWNER" "${names[i]}"
  done
  [[ $any -eq 0 ]] && dim "    (nothing selected)"
  echo
  dim "    source: $OWNER (pinned) · nothing runs before you say yes · network: none"
  printf "  Run these %s?  [y] yes   [n] back   [q] cancel: " "$(count_sel)"
}

install_run(){
  cls; echo "  Installing…"; echo
  local i
  for ((i=0;i<n;i++)); do
    [[ ${sel[i]} -eq 1 ]] || continue
    printf "    > %s-latest   clone · build · register " "${names[i]}"
    sleep 0.4; echo "OK"
    state[i]=installed; sel[i]=0
  done
  echo; dim "    done — press enter"; read -rs
}

while true; do
  browse
  IFS= read -rsn1 key
  if [[ $key == "$E" ]]; then
    IFS= read -rsn2 -t 0.01 rest || rest=""
    case "$rest" in "[A") key=k;; "[B") key=j;; *) key="__esc__";; esac
  fi
  case "$key" in
    k) ((cur=(cur-1+n)%n));;
    j) ((cur=(cur+1)%n));;
    " ") sel[cur]=$((1-sel[cur]));;
    a) for ((i=0;i<n;i++)); do sel[i]=1; done;;
    "") [[ $(count_sel) -eq 0 ]] && continue
        confirm; IFS= read -rsn1 c; echo
        [[ ${c:-} == y ]] && install_run;;
    q|Q) cls; exit 0;;
  esac
done
