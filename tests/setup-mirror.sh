#!/bin/bash

# Exit on error
set -e

UPSTREAM="https://github.com/emlun/gws2.git"
WORKSPACE_DIR="$1"

LOCAL_MIRROR="/tmp/gws2-integration-tests/local-mirror"
LOCAL_MIRROR_AHEAD="/tmp/gws2-integration-tests/local-mirror-ahead"
LOCAL_MIRROR_LOCK="${LOCAL_MIRROR}.lock"


mirror_clone() {
  mkdir -p $(dirname "${LOCAL_MIRROR_LOCK}")
  if [[ -f "${LOCAL_MIRROR_LOCK}" ]]; then
    echo 'Local mirror is locked - skipping.'
  else
    touch "${LOCAL_MIRROR_LOCK}"

    if ! git -C "${LOCAL_MIRROR}" status; then
      rm -rf "${LOCAL_MIRROR}"
      git clone "${UPSTREAM}" "${LOCAL_MIRROR}"
    fi

    if ! git -C "${LOCAL_MIRROR_AHEAD}" status; then
      rm -rf "${LOCAL_MIRROR_AHEAD}"
      git clone "${LOCAL_MIRROR}" "${LOCAL_MIRROR_AHEAD}"
      git -C "${LOCAL_MIRROR_AHEAD}" config commit.gpgSign false
      git -C "${LOCAL_MIRROR_AHEAD}" commit --allow-empty -m "More work"
    fi

    rm -f "${LOCAL_MIRROR_LOCK}"
  fi
}

mirror_clone
