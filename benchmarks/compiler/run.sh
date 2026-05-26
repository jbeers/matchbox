#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
fixture_dir="$script_dir/fixtures"
time_bin="${TIME_BIN:-/usr/bin/time}"
matchbox_release_bin="${MATCHBOX_RELEASE_BIN:-$repo_root/target/release/matchbox}"
matchbox_debug_bin="${MATCHBOX_DEBUG_BIN:-$repo_root/target/debug/matchbox}"
boxlang_bin="${BOXLANG_BIN:-boxlang}"
mode="${1:-both}"

bash "$script_dir/generate-fixtures.sh" >/dev/null

fixture_names=(
    "small"
    "large-generated"
    "function-heavy"
    "class-heavy"
    "template-heavy"
)

fixture_paths=(
    "$fixture_dir/small_script.bxs"
    "$fixture_dir/generated/large_generated.bxs"
    "$fixture_dir/function_heavy.bxs"
    "$fixture_dir/class_heavy.bxs"
    "$fixture_dir/template_heavy.bxm"
)

fixture_expected=(
    "compiler-small: compiler:42"
    "compiler-large: 80200"
    "compiler-functions: 390"
    "compiler-class: 42"
    "compiler-template: OK"
)

engine_available() {
    case "$1" in
        matchbox-release) [[ -x "$matchbox_release_bin" ]] ;;
        matchbox-debug) [[ -x "$matchbox_debug_bin" ]] ;;
        boxlang) command -v "$boxlang_bin" >/dev/null 2>&1 ;;
        *) return 1 ;;
    esac
}

engine_command() {
    case "$1" in
        matchbox-release) printf '%s\n' "$matchbox_release_bin" ;;
        matchbox-debug) printf '%s\n' "$matchbox_debug_bin" ;;
        boxlang) printf '%s\n' "$boxlang_bin" ;;
        *) return 1 ;;
    esac
}

selected_engines() {
    case "$mode" in
        matchbox-release) printf '%s\n' "matchbox-release" ;;
        matchbox-debug) printf '%s\n' "matchbox-debug" ;;
        matchbox) printf '%s\n%s\n' "matchbox-release" "matchbox-debug" ;;
        boxlang) printf '%s\n' "boxlang" ;;
        both) printf '%s\n%s\n' "matchbox-release" "boxlang" ;;
        all) printf '%s\n%s\n%s\n' "matchbox-release" "matchbox-debug" "boxlang" ;;
        *)
            echo "usage: bash benchmarks/compiler/run.sh [matchbox-release|matchbox-debug|matchbox|boxlang|both|all]" >&2
            return 2
            ;;
    esac
}

metric() {
    local label="$1"
    local file="$2"
    awk -F': ' -v label="$label" 'index($0, label) { value=$2 } END { print value }' "$file"
}

had_failure=0

run_fixture() {
    local engine="$1"
    local fixture_name="$2"
    local fixture_path="$3"
    local expected="$4"
    local command
    local out_file
    local err_file
    local status
    command="$(engine_command "$engine")"
    out_file="$(mktemp)"
    err_file="$(mktemp)"

    set +e
    "$time_bin" -v "$command" "$fixture_path" >"$out_file" 2>"$err_file"
    status=$?
    set -e

    local ok="ok"
    if [[ "$status" -ne 0 ]]; then
        ok="exit-$status"
    elif ! grep -Fq "$expected" "$out_file"; then
        ok="bad-output"
    fi

    printf '| %s | %s | %s | %s | %s | %s | %s |\n' \
        "$engine" \
        "$fixture_name" \
        "$(metric "Elapsed (wall clock) time" "$err_file")" \
        "$(metric "User time" "$err_file")" \
        "$(metric "System time" "$err_file")" \
        "$(metric "Maximum resident set size" "$err_file")" \
        "$ok"

    if [[ "$ok" != "ok" ]]; then
        had_failure=1
        echo "--- stdout ($engine $fixture_name) ---" >&2
        cat "$out_file" >&2
        echo "--- stderr/time ($engine $fixture_name) ---" >&2
        cat "$err_file" >&2
    fi

    rm -f "$out_file" "$err_file"
}

if [[ ! -x "$time_bin" ]]; then
    echo "time executable not found: $time_bin" >&2
    exit 2
fi

printf '| engine | fixture | wall | user | sys | max_rss_kb | status |\n'
printf '|---|---|---:|---:|---:|---:|---|\n'

while IFS= read -r engine; do
    if ! engine_available "$engine"; then
        echo "skipping $engine; binary not found" >&2
        continue
    fi
    for idx in "${!fixture_names[@]}"; do
        run_fixture "$engine" "${fixture_names[$idx]}" "${fixture_paths[$idx]}" "${fixture_expected[$idx]}"
    done
done < <(selected_engines)

exit "$had_failure"
