#!/usr/bin/env bash

set -e
set -o pipefail

function check_versions_match {
    local versions=()
    while read -r l; do
        version=$(tomlq -r .package.version "$l")
        versions=("${versions[@]}" "$l":"$version")
    done < <(find . -mindepth 2 -maxdepth 2 -type f -name Cargo.toml | sort)

    version_count=$(echo "${versions[@]}" | xargs -n 1 | cut -d':' -f 2 | sort | uniq | wc -l)
    if [ "$version_count" -ne 1 ]; then
        echo "Package versions don't match:" >&2
        echo >&2
        echo "${versions[@]}" | xargs -n 1 >&2
        echo >&2
        return 1
    fi
}

check_versions_match

function runcheck() {
    cargo test
    cargo clean -p "$(cargo read-manifest | jq -r '.name')"
    cargo clippy -- -D warnings
    cargo fmt -- --check
}

pushd lsdirs
runcheck
popd

pushd freqdirs
runcheck
popd

