# WebAssembly API Gateway

This is an experimental project to test, whether the basic functionality of a plugin-based API Gateway can be implemented in Spin, Wasmtime or WebAssembly components.

## Structure

- `api-gateway`: The main API Gateway component, which is responsible for routing requests to the appropriate plugins.
- `api-go`: _not working yet_ A sample plugin in TinyGo that can be used to test the API Gateway functionality.
- `api-js`: A sample plugin in Javascript that can be used to test the API Gateway functionality.
- `api-rs`: A sample plugin in Rust that can be used to test the API Gateway functionality.

## Links

- analyze for more [performant component loading and caching](https://github.com/dicej/wasmtime-serverless-performance) options
