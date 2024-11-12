FROM denoland/deno:2.0.6 as build

WORKDIR /tmp/app

COPY ./apps/website .

RUN deno install --frozen && deno task build

FROM nginx:alpine

COPY ./apps/website/nginx.conf /etc/nginx/nginx.conf

COPY --from=build /tmp/app/build /var/www

ENTRYPOINT ["nginx", "-g", "daemon off;"]
