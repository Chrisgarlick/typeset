// The browser talks to the server-side proxy at /ui-api, which adds the API
// token. No secret is ever shipped to the browser.
async function apiFetch(path: string, options: RequestInit = {}) {
  const res = await fetch(`/ui-api${path}`, options);

  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || res.statusText);
  }

  return res;
}

// --- Clients ---
// Brands are managed in Clients (https://clients.kritano.com); Typeset lists
// the ones the key may use.

export interface Client {
  slug: string;
  name: string;
  brand_version: number | null;
}

export async function getClients(): Promise<Client[]> {
  const res = await apiFetch("/clients");
  return res.json();
}

// --- Render ---

export interface RenderRequest {
  client?: string;
  /** A published brand version to pin to; the latest when omitted. */
  brand_version?: number;
  document_type: string;
  format: "pdf" | "docx" | "both";
  content: string;
  overrides?: {
    title?: string;
    subtitle?: string;
    recipient?: string;
    date?: string;
    author?: string;
  };
}

export interface RenderResponse {
  success: boolean;
  render_id: string;
  pdf_url: string | null;
  docx_url: string | null;
  expires_at: string;
  render_ms: number;
}

export async function renderDocument(
  req: RenderRequest
): Promise<RenderResponse> {
  const res = await apiFetch("/render", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
  return res.json();
}

export async function previewDocument(
  req: RenderRequest
): Promise<Blob> {
  const res = await apiFetch("/preview", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
  return res.blob();
}

// --- History ---

export interface RenderHistoryItem {
  id: string;
  client_slug: string | null;
  document_type: string;
  format: string;
  status: string;
  pdf_url: string | null;
  docx_url: string | null;
  render_ms: number | null;
  created_at: string;
}

export async function getHistory(
  limit = 20,
  offset = 0
): Promise<RenderHistoryItem[]> {
  const res = await apiFetch(
    `/history?limit=${limit}&offset=${offset}`
  );
  return res.json();
}
