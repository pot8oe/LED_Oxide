FROM rust:1.53

WORKDIR /usr/src/led_oxide
COPY . .

RUN apt-get update && \
apt-get install -y libudev-dev && \
rustup default nightly && \
rustup update && \
cargo install --path .

CMD ["led_oxide"]
