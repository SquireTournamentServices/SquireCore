FROM squire_core_build AS build

# Setup web server (NGINX)
FROM ubuntu

RUN useradd -ms /bin/bash app
USER app
WORKDIR /home/app
COPY --from=build --chown=app /app/SquireCore/target/debug/ .

EXPOSE 8000/tcp

ENTRYPOINT ["./squire_core"]
