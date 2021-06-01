ARG IMAGE_NAME="rust"
ARG BUILD_IMAGE_VERSION="1.52.1-buster"
ARG RUN_IMAGE_VERSION="1.52.1-slim-buster"

FROM ${IMAGE_NAME}:${BUILD_IMAGE_VERSION} AS builder

ARG CARGO_BUILD_ARGS="--release"

WORKDIR /build

COPY . .

RUN apt-get update \
    && apt-get install -y libssl-dev pkg-config \
    && apt-get clean

RUN cargo build ${BUILD_ARGS}

FROM ${IMAGE_NAME}:${RUN_IMAGE_VERSION}

ARG TARGET_TYPE="release"

RUN mkdir -p /app/configs
WORKDIR /app

COPY --from=builder /build/target/${TARGET_TYPE}/develbot /app/

CMD ["./develbot"]
