# syntax=docker/dockerfile:1

# ---------- build stage ----------
# Pinned to an LTS: Node 26 removed buffer.SlowBuffer, which
# buffer-equal-constant-time reads at import time via
# firebase-admin -> jsonwebtoken -> jws -> jwa. See the README.
FROM node:22-bookworm-slim AS builder

WORKDIR /usr/src/auth-server

# npm ci installs exactly what package-lock.json pins and fails if the manifest
# and lockfile disagree; npm i may quietly resolve newer versions instead.
COPY package.json package-lock.json ./
RUN npm ci --omit=dev && npm cache clean --force

# ---------- runtime stage ----------
FROM node:22-bookworm-slim AS runtime

ENV NODE_ENV=production

WORKDIR /usr/src/auth-server

COPY --from=builder /usr/src/auth-server/node_modules ./node_modules
COPY package.json ./
COPY src ./src

EXPOSE 8081

CMD ["node", "src/index.js"]
