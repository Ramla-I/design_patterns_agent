#!/bin/bash

find . -type f -name "*.json" -exec sh -c 'jq "." "$0" > "$0.tmp" && mv "$0.tmp" "$0"' {} \;