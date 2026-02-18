export default {
  async fetch(
    request: Request,
    env: { API_BASE_URL: string; ASSETS: Fetcher },
  ): Promise<Response> {
    const url = new URL(request.url);

    if (url.pathname.startsWith("/api/")) {
      const target = new URL(url.pathname + url.search, env.API_BASE_URL);
      const proxyRequest = new Request(target, request);
      proxyRequest.headers.set("Host", new URL(env.API_BASE_URL).host);
      return fetch(proxyRequest);
    }

    return env.ASSETS.fetch(request);
  },
};
