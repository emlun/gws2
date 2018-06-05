#!/bin/bash

# Exit on error
set -e

UPSTREAM="https://github.com/emlun/gws2.git"
WORKSPACE_DIR="$1"

LOCAL_MIRROR="/tmp/gws2-integration-tests/local-mirror"
LOCAL_MIRROR_AHEAD="/tmp/gws2-integration-tests/local-mirror-ahead"
LOCAL_MIRROR_LOCK="${LOCAL_MIRROR}.lock"
REMOTE2="file://${LOCAL_MIRROR}"

PROJECT_CLEAN="${WORKSPACE_DIR}/clean"
PROJECT_NEW_LOCAL_COMMIT="${WORKSPACE_DIR}/new_commit/local"
PROJECT_NEW_REMOTE_COMMIT="${WORKSPACE_DIR}/new_commit/remote"
PROJECT_NEW_UNFETCHED_REMOTE_COMMIT="${WORKSPACE_DIR}/new_commit/unfetched_remote"
PROJECT_NEW_FILES="${WORKSPACE_DIR}/changes/new_files"
PROJECT_CHANGED_FILES="${WORKSPACE_DIR}/changes/changed_files"


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

project_clean() {
  git clone "${LOCAL_MIRROR}" "${PROJECT_CLEAN}"
  git -C "${PROJECT_CLEAN}" remote add remote2 "${REMOTE2}"
  git -C "${PROJECT_CLEAN}" fetch remote2
  git -C "${PROJECT_CLEAN}" checkout -b master2 remote2/master
}

project_new_local_commit() {
  git clone "${LOCAL_MIRROR}" "${PROJECT_NEW_LOCAL_COMMIT}"
  git -C "${PROJECT_NEW_LOCAL_COMMIT}" config commit.gpgSign false
  git -C "${PROJECT_NEW_LOCAL_COMMIT}" remote add remote2 "${REMOTE2}"
  git -C "${PROJECT_NEW_LOCAL_COMMIT}" fetch remote2
  git -C "${PROJECT_NEW_LOCAL_COMMIT}" checkout -b master2 remote2/master
  git -C "${PROJECT_NEW_LOCAL_COMMIT}" commit --allow-empty -m "More work"
}

project_new_remote_commit() {
  git clone "${LOCAL_MIRROR}" "${PROJECT_NEW_REMOTE_COMMIT}"
  git -C "${PROJECT_NEW_REMOTE_COMMIT}" remote add remote2 "${REMOTE2}"
  git -C "${PROJECT_NEW_REMOTE_COMMIT}" fetch remote2
  git -C "${PROJECT_NEW_REMOTE_COMMIT}" checkout -b master2 remote2/master
  git -C "${PROJECT_NEW_REMOTE_COMMIT}" reset --hard HEAD~
}

project_new_unfetched_remote_commit() {
  git clone "${LOCAL_MIRROR}" "${PROJECT_NEW_UNFETCHED_REMOTE_COMMIT}"
  git -C "${PROJECT_NEW_UNFETCHED_REMOTE_COMMIT}" remote add remote2 "${LOCAL_MIRROR}"
  git -C "${PROJECT_NEW_UNFETCHED_REMOTE_COMMIT}" fetch remote2
  git -C "${PROJECT_NEW_UNFETCHED_REMOTE_COMMIT}" checkout -b master2 remote2/master
  git -C "${PROJECT_NEW_UNFETCHED_REMOTE_COMMIT}" remote set-url remote2 "${REMOTE2}"
}

project_new_files() {
  git clone "${LOCAL_MIRROR}" "${PROJECT_NEW_FILES}"
  git -C "${PROJECT_NEW_FILES}" remote add remote2 "${REMOTE2}"
  git -C "${PROJECT_NEW_FILES}" fetch remote2
  git -C "${PROJECT_NEW_FILES}" checkout -b master2 remote2/master
  touch "${PROJECT_NEW_FILES}/foo.txt"
}

project_changed_files() {
  git clone "${LOCAL_MIRROR}" "${PROJECT_CHANGED_FILES}"
  git -C "${PROJECT_CHANGED_FILES}" remote add remote2 "${REMOTE2}"
  git -C "${PROJECT_CHANGED_FILES}" fetch remote2
  git -C "${PROJECT_CHANGED_FILES}" checkout -b master2 remote2/master
  echo "flrglgrgldrgl" >> "${PROJECT_CHANGED_FILES}/README.md"
}

project_missing_repository() {
  true
}

project_missing_repository_2() {
  true
}

mirror_clone
project_clean
project_new_local_commit
project_new_remote_commit
project_new_unfetched_remote_commit
project_new_files
project_changed_files
project_missing_repository
project_missing_repository_2
