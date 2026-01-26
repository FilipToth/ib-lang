#!/usr/bin/env bash
set -euo pipefail

HOST=filiptoth@104.248.127.109
REMOTE=~/stack

# static sites
( cd ../ide-web && npm run build )
( cd ../landing && npm run build )

rsync -av --delete ../ide-web/build/  "$HOST:$REMOTE/static/iblang-app/"
rsync -av --delete ../landing/build/  "$HOST:$REMOTE/static/iblang-docs/"

# rust services
docker buildx build --platform linux/amd64 -t auth-server:latest ../auth-server
docker buildx build --platform linux/amd64 -t ib-core:latest     ../core

docker save auth-server:latest | gzip | ssh "$HOST" 'gunzip | docker load'
docker save ib-core:latest     | gzip | ssh "$HOST" 'gunzip | docker load'

# copy config
scp Caddyfile docker-compose.yml "$HOST:$REMOTE/"

# restart
ssh "$HOST" "cd $REMOTE && docker compose up -d --force-recreate auth-server ib-core && docker compose up -d caddy"