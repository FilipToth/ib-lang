FROM node:bookworm-slim

ENV NODE_ENV development

WORKDIR /usr/src/auth-server

COPY . .

RUN npm i

EXPOSE 8081

CMD ["node", "src/index.js"]