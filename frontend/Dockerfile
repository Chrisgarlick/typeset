FROM node:22-alpine AS builder

WORKDIR /app

COPY package.json bun.lock ./
RUN npm install

COPY . .

# No secrets at build time: TYPESET_TOKEN and TYPESET_API_URL are read by the
# /ui-api proxy at runtime (see docker-compose files).
RUN npm run build

# Runtime
FROM node:22-alpine

WORKDIR /app

COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
COPY --from=builder /app/public ./public

ENV PORT=3000
ENV HOSTNAME=0.0.0.0
EXPOSE 3000

CMD ["node", "server.js"]
