#!/usr/bin/env bash
# Usage: ./psql.sh -f test_data.sql
set -euo pipefail

source '.env'

postgres_args=(
  --variable=ON_ERROR_STOP=1
  --dbname "${POSTGRES_DB}"
  --host 127.0.0.1
  --username "${POSTGRES_USER}"
)

psql "${postgres_args[@]}" "${@}"
