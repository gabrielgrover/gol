FROM rust as builder

COPY . /app

WORKDIR /app

RUN cargo build --release

FROM gcr.io/distroless/cc-debian11

COPY --from=builder /app/target/release/game_of_life_server /app/game_of_life_server

WORKDIR /app

CMD ["./game_of_life_server"]
