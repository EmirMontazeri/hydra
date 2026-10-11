#!/usr/bin/env bash
set -euo pipefail
SIGNPATH_TEST_SIGNING=${SIGNPATH_TEST_SIGNING:-true}
case "$SIGNPATH_TEST_SIGNING" in
  true|false) echo "test_signing=$SIGNPATH_TEST_SIGNING" >> "$GITHUB_OUTPUT" ;;
  *) echo '::error::SIGNPATH_TEST_SIGNING must be true or false'; exit 1 ;;
esac
