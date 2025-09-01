#!/usr/bin/env bash

set -ex

main() {
    local package_name="$CRATE_NAME-${GITHUB_REF_NAME#v}-$TARGET"

    local staging_prefix
    staging_prefix=$(mktemp -d)

    local working_prefix=$staging_prefix/$package_name

    local dst_prefix
    dst_prefix=$(pwd)

    mkdir "$working_prefix"

    cp "target/$TARGET/release/$CRATE_NAME" "$working_prefix"
    cp LICENSE.txt README.md "$working_prefix"

    pushd "$staging_prefix" 1>&2

    if [[ $OS_NAME == windows-* ]]; then
        dst="$dst_prefix/$package_name.zip"
        7z a "$dst" "$package_name" 1>&2
    else
        dst="$dst_prefix/$package_name.tar.gz"
        tar cfz "$dst" "$package_name"
    fi

    popd 1>&2

    rm -r "$staging_prefix"

    echo "$dst"
}

main
