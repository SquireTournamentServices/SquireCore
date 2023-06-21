FROM squire_core_build AS build

# Setup web server (NGINX)
FROM nginx:alpine

WORKDIR /usr/share/nginx/html
RUN rm -rf ./*
COPY --from=build --chown=nginx /app/SquireCore/squire_web/dist/ .

EXPOSE 80/tcp

ENTRYPOINT ["nginx", "-g", "daemon off;"]
