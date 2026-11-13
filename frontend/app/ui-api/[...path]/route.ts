// Server-side proxy from the browser to the Rust API. The API token is read
// from the server's environment and added here, so it never reaches the
// browser bundle. nginx puts this route behind basic auth (see
// typeset.nginx.conf) — without that, it would let anyone use the API.

const API_URL = process.env.TYPESET_API_URL || "http://localhost:3200";

// Only the endpoints the UI uses.
const ALLOWED = new Set(["clients", "render", "preview", "proof", "history", "health"]);

// Response headers worth passing back to the browser.
const PASS_HEADERS = ["content-type", "content-disposition", "retry-after"];

async function proxy(
  request: Request,
  { params }: { params: Promise<{ path: string[] }> }
) {
  const { path } = await params;
  if (!ALLOWED.has(path[0])) {
    return Response.json({ error: "Not found", status: 404 }, { status: 404 });
  }

  const token = process.env.TYPESET_TOKEN;
  if (!token) {
    return Response.json(
      { error: "TYPESET_TOKEN is not set on the frontend server", status: 500 },
      { status: 500 }
    );
  }

  const url = new URL(request.url);
  const target = `${API_URL}/api/${path.map(encodeURIComponent).join("/")}${url.search}`;

  const headers: Record<string, string> = { Authorization: `Bearer ${token}` };
  const contentType = request.headers.get("content-type");
  if (contentType) headers["Content-Type"] = contentType;
  // Keep per-IP rate limiting meaningful: forward the caller's address from nginx.
  const realIp = request.headers.get("x-real-ip");
  if (realIp) headers["X-Real-IP"] = realIp;

  const hasBody = request.method !== "GET" && request.method !== "HEAD";
  const upstream = await fetch(target, {
    method: request.method,
    headers,
    body: hasBody ? await request.arrayBuffer() : undefined,
    cache: "no-store",
  });

  const out = new Headers();
  for (const name of PASS_HEADERS) {
    const value = upstream.headers.get(name);
    if (value) out.set(name, value);
  }
  return new Response(upstream.body, { status: upstream.status, headers: out });
}

export { proxy as GET, proxy as POST, proxy as DELETE };
