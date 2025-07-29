.PHONY: all api1 api2 api-gateway run

all: api1 api2 api-gateway

api1:
	cd api1
	cargo build -p api1 --target wasm32-wasip2

api2:
	cd api2
	cargo build -p api2 --target wasm32-wasip2

api-gateway:
	cd api-gateway
	cargo build -p api-gateway

run:
	target/debug/api-gateway
