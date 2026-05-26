#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out_dir="$script_dir/fixtures/generated"
out_file="$out_dir/large_generated.bxs"

mkdir -p "$out_dir"

{
    echo "// Generated compiler benchmark fixture. Regenerate with benchmarks/compiler/generate-fixtures.sh."
    echo "total = 0"
    for i in $(seq 1 400); do
        echo "total = total + $i"
    done
    echo 'println("compiler-large: " & total)'
} > "$out_file"

echo "$out_file"
