FROM node:23 AS build

RUN npm install -g pnpm

WORKDIR /tmp/app

COPY ./apps/website .

# pnpm stops asking questions when CI is set to 1
ENV CI=1

RUN pnpm install && pnpm run build

FROM nginx:alpine

COPY ./apps/website/nginx.conf /etc/nginx/nginx.conf

COPY --from=build /tmp/app/build /var/www

ENTRYPOINT ["nginx", "-g", "daemon off;"]
