.PHONY: all api1 api2 api-gateway run

all: api1 api2 api-gateway

api1:
	cd api1
	cargo build -p api1 --target wasm32-wasip2
	# cargo build -p api1 --release --target wasm32-unknown-unknown
	# wasm-tools component new target/wasm32-unknown-unknown/release/api1.wasm \
	# 	-o target/release/api1.component.wasm

api2:
	cd api2 && cargo component build --target wasm32-unknown-unknown

api-gateway:
	cd api-gateway
	cargo build -p api-gateway --release

run:
	cd api-gateway
	cargo run
