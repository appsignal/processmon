#!/bin/sh

echo "Run trigger (triggered by $TRIGGER_PATH)"
echo "$TRIGGER_PATH - $CONTENT_FOR_FILE" > triggered.txt
