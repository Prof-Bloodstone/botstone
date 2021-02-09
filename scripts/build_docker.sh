#!/usr/bin/env bash

set -euo pipefail

root_dir="$(git rev-parse --show-toplevel)"
readonly root_dir
cd "${root_dir}"
commit_sha="$(git log -n1 --format=%h)"
readonly commit_sha
readonly relative_image_location="target/botstone.docker.image.tar"

if git diff --quiet --exit-code HEAD &>/dev/null; then
  readonly dirty="F"
else
  readonly dirty="D"
fi

readonly img_tag="${dirty}${commit_sha}"
readonly img_name="botstone"
readonly img_full_name="${img_name}:${img_tag}"

docker build -t "${img_full_name}" "${root_dir}"
docker tag "${img_full_name}" "${img_name}:latest"

docker image save "${img_full_name}" --output "${root_dir}/${relative_image_location}"

printf 'Successfully built docker image as %s and saved it to %s\n' "${img_full_name@Q}" "${relative_image_location@Q}"

