# Makefile for building api components and the api-gateway

.PHONY: all api1 api2 api-gateway run

all: api1 api2 api-gateway

api1:
        cd api1 && cargo component build --target wasm32-unknown-unknown

api2:
        cd api2 && cargo component build --target wasm32-unknown-unknown

api-gateway:
	cd api-gateway && spin build

run:
	cd api-gateway && spin up
