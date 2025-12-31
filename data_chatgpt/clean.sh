#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ]; then
    echo "Usage: $0 <path/to/conversations.json>"
    exit 1
fi

INPUT_FILE="$1"
FILE_NAME=$(basename "$INPUT_FILE" .json)

JSONL_FILE="${FILE_NAME}.jsonl"
CLEAN_FILE="${FILE_NAME}_clean.jsonl"


echo "Converting $INPUT_FILE to JSONL → $JSONL_FILE..."
jq --stream -c 'fromstream(1|truncate_stream(inputs))' "$INPUT_FILE" > "$JSONL_FILE"

echo "Cleaning conversations → $CLEAN_FILE..."
jq -c '
select(.title != null and .title != "")
| {
    title: .title,
    messages: (
      .mapping
      | to_entries
      | map(.value.message)
      | map(select(. != null))
      | map(select(.content.content_type == "text"))
      | map({
          role: .author.role,
          parts: (.content.parts | join(""))
        })
      | map(select(.parts | length > 0))
    )
  }
' "$JSONL_FILE" > "$CLEAN_FILE"

echo "Cleaned conversations written to $CLEAN_FILE"
