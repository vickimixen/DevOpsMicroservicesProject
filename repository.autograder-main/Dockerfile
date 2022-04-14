FROM rustlang/rust:nightly as build-stage
WORKDIR /usr/src/repository-microservice
COPY . .
RUN cargo install --path .

FROM debian:buster-slim as production-stage
RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*
COPY --from=build-stage /usr/local/cargo/bin/repository-microservice /usr/local/bin/repository-microservice
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8888
CMD ["repository-microservice"]
