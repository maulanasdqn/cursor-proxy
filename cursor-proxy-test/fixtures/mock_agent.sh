#!/bin/bash

# Mock agent CLI for integration tests.
# Responds based on arguments to simulate Cursor agent behavior.

for arg in "$@"; do
    if [ "$arg" = "--list-models" ]; then
        echo "opus-4.6  -  Claude 4.6 Opus (recommended)"
        echo "sonnet-4.6  -  Claude 4.6 Sonnet (fast)"
        echo "sonnet-4.5  -  Claude 4.5 Sonnet (legacy)"
        echo "opus-4.6-thinking  -  Claude 4.6 Opus Thinking"
        echo "sonnet-4.6-thinking  -  Claude 4.6 Sonnet Thinking"
        echo "sonnet-4.5-thinking  -  Claude 4.5 Sonnet Thinking"
        exit 0
    fi
done

STREAM=false
OUTPUT_FORMAT="text"

while [ $# -gt 0 ]; do
    case "$1" in
        --stream-partial-output)
            STREAM=true
            shift
            ;;
        --output-format)
            OUTPUT_FORMAT="$2"
            shift 2
            ;;
        --print|--trust|--force|--approve-mcps)
            shift
            ;;
        --mode|--workspace|--model)
            shift 2
            ;;
        *)
            PROMPT="$1"
            shift
            ;;
    esac
done

if [ "$STREAM" = true ] && [ "$OUTPUT_FORMAT" = "stream-json" ]; then
    echo '{"type":"assistant","message":{"content":[{"type":"text","text":"Hello from "}]}}'
    echo '{"type":"assistant","message":{"content":[{"type":"text","text":"mock agent"}]}}'
    echo '{"type":"result","subtype":"success"}'
    exit 0
fi

echo "Hello from mock agent"
exit 0
