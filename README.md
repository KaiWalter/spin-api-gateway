# WebAssembly API Gateway

This is an experimental project to test, whether the basic functionality of a plugin-based API Gateway can be implemented in Spin, Wasmtime or WebAssembly components.

## Structure

- `api-gateway`: The main API Gateway component, which is responsible for routing requests to the appropriate plugins.
- `api1`: A sample plugin that can be used to test the API Gateway functionality.
- `api2`: Another sample plugin that can be used to test the API Gateway functionality and validate routing.
