#!/usr/bin/env bash

set -euo pipefail
set -x
sqlx migrate info || true
sqlx database drop -y || true
sqlx database create
sqlx migrate run
