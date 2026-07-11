#!/bin/sh
set -e
chown -R minimamemosa:minimamemosa /app/data
exec su-exec minimamemosa "$@"
