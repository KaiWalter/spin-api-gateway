.PHONY: all api1 api2 guest api-gateway run

all: api1 api2 api-gateway

api1:
	cd api1 && cargo component build --target wasm32-unknown-unknown

api2:
	cd api2 && cargo component build --target wasm32-unknown-unknown

guest:
	cd api-gateway/wasm && cargo build --target wasm32-unknown-unknown

api-gateway:
	cd api-gateway && cargo build

run:
	target/debug/api-gateway
