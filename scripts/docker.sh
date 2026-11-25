#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

# The course image is normally referenced by its digest (see .devcontainer/).
# A `:latest` tag is used here so that a locally imported/tagged image works
# without network access. Tag it once with:
#   docker tag 1301182193/arceos-env:latest docker.io/1301182193/arceos-env:latest
image=docker.io/1301182193/arceos-env:latest

exec docker run --pull=missing --rm -it --platform linux/amd64 --user root --hostname arceos \
    -v "$PWD:/workspace" "$image" "$@"
