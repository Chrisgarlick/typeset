// In the browser, use relative paths (Next.js rewrites /api/* to the Rust service).
// On the server or if NEXT_PUBLIC_API_URL is set, use that directly.
const API_URL =
  typeof window !== "undefined"
    ? ""
    : process.env.NEXT_PUBLIC_API_URL || "http://localhost:3200";

// Single shared secret. Must match the TYPESET_TOKEN env var on the API.
// Set NEXT_PUBLIC_API_TOKEN at build time in production.
const API_TOKEN = process.env.NEXT_PUBLIC_API_TOKEN || "dev-token";

async function apiFetch(path: string, options: RequestInit = {}) {
  const res = await fetch(`${API_URL}${path}`, {
    ...options,
    headers: {
      Authorization: `Bearer ${API_TOKEN}`,
      ...options.headers,
    },
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error || res.statusText);
  }

  return res;
}

// --- Client Profiles ---

export interface ClientProfile {
  id: string;
  slug: string;
  name: string;
  colour_primary: string;
  colour_secondary: string;
  colour_accent: string;
  colour_text: string;
  colour_background: string;
  colour_table_header: string;
  colour_table_border: string;
  colour_callout_bg: string;
  font_heading: string;
  font_body: string;
  font_mono: string;
  font_size_base: number;
  font_size_h1: number;
  font_size_h2: number;
  font_size_h3: number;
  line_height: number;
  page_size: string;
  margin_top: number;
  margin_bottom: number;
  margin_left: number;
  margin_right: number;
  paragraph_spacing: number;
  section_spacing: number;
  logo_light_url: string | null;
  logo_dark_url: string | null;
  logo_width: number;
  logo_position: string;
  cover_enabled: boolean;
  cover_template: string;
  cover_bg_colour: string;
  cover_text_colour: string;
  header_enabled: boolean;
  header_template: string;
  header_border: boolean;
  footer_enabled: boolean;
  footer_template: string;
  footer_border: boolean;
  watermark_text: string | null;
  watermark_opacity: number;
}

export async function getClients(): Promise<ClientProfile[]> {
  const res = await apiFetch("/api/clients");
  return res.json();
}

export async function getClient(slug: string): Promise<ClientProfile> {
  const res = await apiFetch(`/api/clients/${slug}`);
  return res.json();
}

export async function saveClient(
  profile: Partial<ClientProfile> & { slug: string; name: string }
): Promise<ClientProfile> {
  const res = await apiFetch("/api/clients", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(profile),
  });
  return res.json();
}

export async function deleteClient(slug: string): Promise<void> {
  await apiFetch(`/api/clients/${slug}`, { method: "DELETE" });
}

// --- Render ---

export interface RenderRequest {
  client?: string;
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
  const res = await apiFetch("/api/render", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
  return res.json();
}

export async function previewDocument(
  req: RenderRequest
): Promise<Blob> {
  const res = await apiFetch("/api/preview", {
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
    `/api/history?limit=${limit}&offset=${offset}`
  );
  return res.json();
}
