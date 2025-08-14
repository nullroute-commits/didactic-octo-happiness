#!/usr/bin/env bash
# Usage: 50_uptime_info.sh <arch>
ARCH="$1"

if [ -z "$ARCH" ]; then
  echo "Error: architecture parameter is required." >&2
  echo "Usage: $0 <arch>" >&2
  exit 1
fi
if [ -r /proc/uptime ] && [ -f /proc/uptime ]; then
  uptime=$(awk '{print int($1)}' /proc/uptime)
else
  uptime="unknown"
  echo "Warning: /proc/uptime is not readable or does not exist." >&2
fi
UP_JSON=$(jq -n \
  --arg arch "$ARCH" \
  --arg uptime "$uptime" \
  '{architecture: $arch, uptime_seconds: $uptime}')

echo "$UP_JSON"