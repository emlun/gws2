#!/bin/sh

set -ex

KEYFILE=tests/id_rsa
GPG_KEYGRIP=000B8A4DF023522536010F9B4E6546DCB7E01A1C

ssh-add -d tests/id_rsa

if gpg-connect-agent /bye; then
  echo "DELETE_KEY --force ${GPG_KEYGRIP}" | gpg-connect-agent
fi
