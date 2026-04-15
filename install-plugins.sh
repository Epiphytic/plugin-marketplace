#!/usr/bin/env bash
set -euo pipefail

# Epiphytic Plugin Installer
# Interactive plugin selection and installation from registry.yaml
#
# Usage: ./install-plugins.sh [--non-interactive]

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REGISTRY="$SCRIPT_DIR/registry.yaml"

# Restore terminal on exit/interrupt
cleanup() {
	printf '\033[?25h' 2>/dev/null
	stty sane 2>/dev/null
}
trap 'cleanup; exit 130' INT TERM
trap cleanup EXIT

# --- Color output helpers ---
info() { printf '\033[0;34m[INFO]\033[0m  %s\n' "$1"; }
warn() { printf '\033[0;33m[WARN]\033[0m  %s\n' "$1"; }
error() { printf '\033[0;31m[ERROR]\033[0m %s\n' "$1"; }
ok() { printf '\033[0;32m[OK]\033[0m    %s\n' "$1"; }

# --- Dependency check ---

check_dependencies() {
	if ! command -v yq &>/dev/null; then
		error "yq is required but not found."
		echo "  Install with: brew install yq (macOS) or see https://github.com/mikefarah/yq"
		exit 1
	fi

	if ! command -v claude &>/dev/null; then
		error "Claude Code is required but not found."
		echo "  Install Claude Code first: https://docs.anthropic.com/en/docs/claude-code"
		exit 1
	fi

	if [[ ! -f "$REGISTRY" ]]; then
		error "registry.yaml not found at $REGISTRY"
		exit 1
	fi
}

# --- Registry reading ---

# Read plugins by tier into parallel arrays
# Sets: NAMES[], DESCRIPTIONS[], INSTALL_CMDS[]
read_plugins_by_tier() {
	local tier="$1"
	NAMES=()
	DESCRIPTIONS=()
	INSTALL_CMDS=()

	local count
	count=$(yq "[.plugins[] | select(.default_install == \"$tier\")] | length" "$REGISTRY")

	for ((i = 0; i < count; i++)); do
		NAMES+=("$(yq "[.plugins[] | select(.default_install == \"$tier\")].[${i}].name" "$REGISTRY")")
		DESCRIPTIONS+=("$(yq "[.plugins[] | select(.default_install == \"$tier\")].[${i}].description" "$REGISTRY")")
		INSTALL_CMDS+=("$(yq "[.plugins[] | select(.default_install == \"$tier\")].[${i}].install" "$REGISTRY")")
	done
}

# --- Interactive checkbox UI ---

# Renders a checkbox list and lets user toggle items with arrow keys + space
# Args: $1 = prompt text, $2 = 1 if items should be pre-selected
# Uses: NAMES[], DESCRIPTIONS[] (read before calling)
# Sets: SELECTED[] (0 or 1 for each item)
checkbox_select() {
	local prompt="$1"
	local preselect="${2:-0}"
	local count=${#NAMES[@]}

	if [[ $count -eq 0 ]]; then
		return
	fi

	# Initialize selection state
	SELECTED=()
	for ((i = 0; i < count; i++)); do
		SELECTED+=("$preselect")
	done

	local cursor=0

	# Hide cursor
	printf '\033[?25l'

	# Print header
	echo ""
	printf '\033[1m%s\033[0m\n' "$prompt"
	echo "  ↑/↓ navigate  ·  Space toggle  ·  Enter confirm"
	echo ""

	# Draw initial list
	_draw_list "$cursor" "$count"

	# Input loop
	while true; do
		local key
		IFS= read -rsn1 key

		case "$key" in
		$'\033')
			# Escape sequence — read next two chars
			local seq
			IFS= read -rsn2 seq
			case "$seq" in
			'[A') # Up arrow
				if ((cursor > 0)); then
					((cursor--)) || true
				fi
				;;
			'[B') # Down arrow
				if ((cursor < count - 1)); then
					((cursor++)) || true
				fi
				;;
			esac
			;;
		' ')
			# Space — toggle selection
			if [[ "${SELECTED[$cursor]}" == "1" ]]; then
				SELECTED[$cursor]=0
			else
				SELECTED[$cursor]=1
			fi
			;;
		'')
			# Enter — confirm
			break
			;;
		esac

		# Move cursor up to redraw list
		printf "\033[${count}A"
		_draw_list "$cursor" "$count"
	done

	# Show cursor
	printf '\033[?25h'
	echo ""
}

_draw_list() {
	local cursor=$1
	local count=$2

	for ((i = 0; i < count; i++)); do
		local marker="[ ]"
		if [[ "${SELECTED[$i]}" == "1" ]]; then
			marker="[x]"
		fi

		local prefix="  "
		if [[ $i -eq $cursor ]]; then
			prefix="> "
		fi

		# Truncate description to fit terminal width
		local desc="${DESCRIPTIONS[$i]}"
		local name="${NAMES[$i]}"
		local cols
		cols=$(tput cols 2>/dev/null || echo 80)
		local max_desc=$((cols - ${#name} - 12))
		if [[ ${#desc} -gt $max_desc ]] && [[ $max_desc -gt 3 ]]; then
			desc="${desc:0:$((max_desc - 3))}..."
		fi

		if [[ $i -eq $cursor ]]; then
			# Highlight current line
			printf '\033[7m%s %s %s — %s\033[0m\033[K\n' "$prefix" "$marker" "$name" "$desc"
		else
			printf '%s %s %s — %s\033[K\n' "$prefix" "$marker" "$name" "$desc"
		fi
	done
}

# --- Installation ---

install_plugins() {
	local -a names=("${!1}")
	local -a cmds=("${!2}")
	local -a selected=("${!3}")
	local count=${#names[@]}

	local installed=0
	local failed=0

	for ((i = 0; i < count; i++)); do
		if [[ "${selected[$i]}" == "1" ]]; then
			local name="${names[$i]}"
			local cmd="${cmds[$i]}"
			info "Installing $name..."
			if eval "$cmd" 2>/dev/null; then
				ok "$name installed"
				((installed++)) || true
			else
				warn "Failed to install $name (command: $cmd)"
				((failed++)) || true
			fi
		fi
	done

	if [[ $installed -gt 0 ]]; then
		ok "$installed plugin(s) installed"
	fi
	if [[ $failed -gt 0 ]]; then
		warn "$failed plugin(s) failed to install"
	fi
}

# --- Main ---

main() {
	echo ""
	echo "╔══════════════════════════════════════╗"
	echo "║     Epiphytic Plugin Installer       ║"
	echo "╚══════════════════════════════════════╝"
	echo ""

	check_dependencies

	local non_interactive=false
	if [[ "${1:-}" == "--non-interactive" ]] || [[ ! -t 0 ]]; then
		non_interactive=true
	fi

	# --- Phase 1: Required plugins (always auto-install) ---
	info "Checking required plugins..."
	read_plugins_by_tier "required"
	local req_count=${#NAMES[@]}

	if [[ $req_count -gt 0 ]]; then
		# All required plugins are selected
		local req_selected=()
		for ((i = 0; i < req_count; i++)); do
			req_selected+=("1")
		done

		local req_names=("${NAMES[@]}")
		local req_cmds=("${INSTALL_CMDS[@]}")

		install_plugins req_names[@] req_cmds[@] req_selected[@]
	else
		ok "No required plugins defined"
	fi

	# --- Non-interactive: stop here ---
	if $non_interactive; then
		echo ""
		info "Non-interactive mode — only required plugins installed."
		info "Run ./install-plugins.sh interactively to select optional plugins."
		return
	fi

	# --- Phase 2: Recommended plugins (pre-selected) ---
	read_plugins_by_tier "recommended"
	local rec_count=${#NAMES[@]}

	if [[ $rec_count -gt 0 ]]; then
		local rec_names=("${NAMES[@]}")
		local rec_descs=("${DESCRIPTIONS[@]}")
		local rec_cmds=("${INSTALL_CMDS[@]}")

		checkbox_select "Recommended plugins (pre-selected):" 1

		install_plugins rec_names[@] rec_cmds[@] SELECTED[@]
	fi

	# --- Phase 3: Optional plugins (none selected) ---
	read_plugins_by_tier "optional"
	local opt_count=${#NAMES[@]}

	if [[ $opt_count -gt 0 ]]; then
		local opt_names=("${NAMES[@]}")
		local opt_descs=("${DESCRIPTIONS[@]}")
		local opt_cmds=("${INSTALL_CMDS[@]}")

		checkbox_select "Optional plugins (none selected by default):" 0

		# Check if any were selected
		local any_selected=false
		for s in "${SELECTED[@]}"; do
			if [[ "$s" == "1" ]]; then
				any_selected=true
				break
			fi
		done

		if $any_selected; then
			install_plugins opt_names[@] opt_cmds[@] SELECTED[@]
		else
			info "No optional plugins selected"
		fi
	fi

	echo ""
	ok "Plugin installation complete!"
	echo "  Use '/plugin' inside Claude Code to manage plugins at any time."
	echo ""
}

main "$@"
