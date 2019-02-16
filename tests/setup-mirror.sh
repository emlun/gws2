#!/bin/bash

# Exit on error
set -e

UPSTREAM="."

LOCAL_MIRROR="/tmp/gws2-integration-tests/local-mirror"
LOCAL_MIRROR_AHEAD="/tmp/gws2-integration-tests/local-mirror-ahead"
LOCAL_MIRROR_LOCK="${LOCAL_MIRROR}.lock"


mirror_clone() {
  mkdir -p $(dirname "${LOCAL_MIRROR_LOCK}")
  if [[ -f "${LOCAL_MIRROR_LOCK}" ]]; then
    echo 'Local mirror is locked - skipping.'
  else
    touch "${LOCAL_MIRROR_LOCK}"

    if [[ -d "${LOCAL_MIRROR}/.git" ]]; then
      echo "Mirror already exists in ${LOCAL_MIRROR}" >&2
      exit 1
    else
      rm -rf "${LOCAL_MIRROR}"
      git clone --branch master "${UPSTREAM}" "${LOCAL_MIRROR}"
    fi

    if [[ -d "${LOCAL_MIRROR_AHEAD}/.git" ]]; then
      echo "Mirror already exists in ${LOCAL_MIRROR_AHEAD}" >&2
      exit 1
    else
      rm -rf "${LOCAL_MIRROR_AHEAD}"
      git clone --branch master "${UPSTREAM}" "${LOCAL_MIRROR_AHEAD}"
      git -C "${LOCAL_MIRROR_AHEAD}" config commit.gpgSign false
      git -C "${LOCAL_MIRROR_AHEAD}" commit --allow-empty -m "More work"
    fi

    rm -f "${LOCAL_MIRROR_LOCK}"
  fi
}

mirror_clone
