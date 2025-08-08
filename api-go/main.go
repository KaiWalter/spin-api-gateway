package main

import (
	"api-go/bindings/gateway/api/api/guest"
	httphandler "api-go/bindings/gateway/api/http-handler"
)

func init() {
	// Set up the exported functions using the generated bindings
	guest.Exports.HandleAPIRequest = func(request httphandler.APIRequest) httphandler.APIRequest {
		// Clone the request and modify it similar to the Rust implementation
		modifiedRequest := request
		modifiedRequest.Host = "https://httpbin.org"
		modifiedRequest.Path = "/get"
		
		return modifiedRequest
	}

	guest.Exports.HandleAPIResponse = func(response httphandler.APIResponse) httphandler.APIResponse {
		// Return the response as-is, similar to other implementations
		return response
	}
}

// main is required for the `wasip2` target, even if it isn't used.
func main() {} 