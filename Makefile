.PHONY: all api1 api2 api-gateway api-js run

all: api1 api2 api-js api-gateway

api1:
	cargo build -p api1 --target wasm32-wasip2

api2:
	cargo build -p api2 --target wasm32-wasip2

api-js:
	cd api-js && npm run build

api-gateway:
	cargo build -p api-gateway

run:
	target/debug/api-gateway
