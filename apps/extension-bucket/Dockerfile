FROM nginx:alpine
WORKDIR /app
COPY ./nginx.conf /etc/nginx/nginx.conf
ENTRYPOINT ["nginx", "-g", "daemon off;"]