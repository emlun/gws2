#!/bin/bash

git_version=$(git describe --tags --always --match='*.*.*' --dirty=-DIRTY)

sed -i 's/\(version\s*=\s*\)".*"/\1"'"${git_version}"'"/' Cargo.toml
