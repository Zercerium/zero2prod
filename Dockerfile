FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:aarch64-musl as builder-for-linux-arm64
ARG TARGET=aarch64-unknown-linux-musl
FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:x86_64-musl as builder-for-linux-amd64
ARG TARGET=x86_64-unknown-linux-musl

FROM builder-for-$BUILDOS-$TARGETARCH as chef
# https://github.com/rui314/mold?tab=readme-ov-file#compile-mold
RUN git clone https://github.com/rui314/mold.git \
    && mkdir mold/build \
    && cd mold/build \
    && git checkout v2.4.1 \
    && ../install-build-deps.sh \
    && cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_CXX_COMPILER=c++ .. \
    && cmake --build . -j $(nproc) \
    && sudo cmake --build . --target install
RUN cargo install cargo-chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM planner as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --locked --release --bin zero2prod
RUN mv /app/target/$TARGET/release/zero2prod /app/target/release/zero2prod
RUN musl-strip /app/target/release/zero2prod

FROM --platform=$TARGETPLATFORM alpine:3 as runtime
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT=production
CMD ["./zero2prod"]
