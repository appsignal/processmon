#!/bin/sh

echo "Run trigger (triggered by $TRIGGER_PATH)"
echo $CONTENT_FOR_FILE > triggered.txt
