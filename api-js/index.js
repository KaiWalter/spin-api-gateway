export const httpHandler = {
    async handleApiRequest(request) {
        return {
            status: 200,
            headers: [["content-type", "text/plain"]],
            body: (request.body ? new TextEncoder().encode(`Hello from api-js! You sent: ${new TextDecoder().decode(request.body)}`) : null)
        };
    }
};
