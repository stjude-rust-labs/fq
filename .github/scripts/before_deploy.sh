#!/usr/bin/env bash

set -ex

main() {
    local package_name="$CRATE_NAME-${GITHUB_REF/refs\/tags\/v/}-$TARGET"

    local staging_prefix
    staging_prefix=$(mktemp -d)

    local working_prefix=$staging_prefix/$package_name

    local dst_prefix
    dst_prefix=$(pwd)

    mkdir "$working_prefix"

    cp "target/$TARGET/release/$CRATE_NAME" "$working_prefix"
    cp LICENSE.txt README.md "$working_prefix"

    pushd "$staging_prefix"

    if [[ $OS_NAME == windows-* ]]; then
        7z a "$dst_prefix/$package_name.zip" "$package_name"
    else
        tar cfz "$dst_prefix/$package_name.tar.gz" "$package_name"
    fi

    popd

    rm -r "$staging_prefix"
}

main
