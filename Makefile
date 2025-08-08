.PHONY: all api-rs api-gateway api-js api-go run

all: api-rs api-js api-go api-gateway

api-rs:
	cargo build -p api-rs --target wasm32-wasip2

api-js:
	cd api-js && npm run build

api-go:
	wkg wit build -d wit -o ./target/api-wit.wasm
	cd api-go && wit-bindgen-go generate -w api -o bindings ../target/api-wit.wasm
	cd api-go && tinygo build -target=wasip2 -o ../target/wasm32-wasip2/debug/api-go.component.wasm main.go

api-gateway:
	cargo build -p api-gateway

run:
	cargo run -p api-gateway
