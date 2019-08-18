#!/bin/sh

set -ex

# Read from stdin so ssh-add doesn't complain about file permissions
ssh-add - < tests/id_rsa
