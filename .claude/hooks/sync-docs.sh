#!/bin/bash
# PostToolUse hook: detect when an English or Chinese doc is modified
# and remind Claude to sync the counterpart.

FILE=$(cat | jq -r '.tool_input.file_path // .tool_input.filePath // empty' 2>/dev/null)

if [ -z "$FILE" ]; then
  exit 0
fi

# Normalize path separators
FILE=$(echo "$FILE" | sed 's|\\|/|g')

# Only care about files under skills/*/references/
if [[ ! "$FILE" =~ skills/.*/references/ ]]; then
  exit 0
fi

BASENAME=$(basename "$FILE")
DIR=$(dirname "$FILE")

# Determine if this is a Chinese or English doc and find its counterpart
COUNTERPART=""
DIRECTION=""

if [[ "$BASENAME" =~ -zh\.md$ ]]; then
  # Chinese doc changed -> find English counterpart
  EN_NAME=$(echo "$BASENAME" | sed 's/-zh\.md$/.md/')
  if [ -f "$DIR/$EN_NAME" ]; then
    COUNTERPART="$DIR/$EN_NAME"
    DIRECTION="Chinese doc changed. English counterpart needs sync"
  fi
elif [[ "$BASENAME" =~ \.md$ ]]; then
  # English doc changed -> find Chinese counterpart
  ZH_NAME=$(echo "$BASENAME" | sed 's/\.md$/-zh.md/')
  if [ -f "$DIR/$ZH_NAME" ]; then
    COUNTERPART="$DIR/$ZH_NAME"
    DIRECTION="English doc changed. Chinese counterpart needs sync"
  fi
fi

if [ -n "$COUNTERPART" ]; then
  echo "[$DIRECTION]"
  echo "Modified: $FILE"
  echo "Please also update: $COUNTERPART"
fi

exit 0
