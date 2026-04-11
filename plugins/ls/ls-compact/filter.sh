#!/bin/sh
# ls-compact — compact ls output for LLM contexts
# env: $LOWFAT_LEVEL (lite|full|ultra)

RAW=$(cat)
LEVEL="${LOWFAT_LEVEL:-full}"

case "$LEVEL" in
  ultra)
    # Filenames only
    echo "$RAW" | grep -v '^total ' | grep -v '^$' | awk '{print $NF}' | head -n 40
    ;;
  *)
    # Strip "total" line
    echo "$RAW" | grep -v '^total ' | grep -v '^$' | head -n 40
    ;;
esac
