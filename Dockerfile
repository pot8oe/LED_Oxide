FROM rust:1.54

WORKDIR /usr/src/led_oxide
COPY . .

RUN apt-get update && \
apt-get install -y libudev-dev libusb-dev && \
rustup default nightly && \
rustup update && \
cargo install --path . && \
cd lib/teensy_loader_cli && make && cd ../..

CMD ["led_oxide"]
