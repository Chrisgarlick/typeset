"use client";

import { useState, useEffect, useCallback, useMemo } from "react";
import { parse as parseYaml } from "yaml";
import { Nav } from "@/components/nav";
import { previewDocument, getClients, TYPE_LABELS, type Client } from "@/lib/api";

const DEFAULT_MD = `---
title: Project Proposal
subtitle: Q3 Strategy
author: Chris Garlick
date: ${new Date().toISOString().split("T")[0]}
---

# Executive Summary

Write your document here using **Markdown**. The brand profile you select will be applied to the exported PDF and DOCX.

## Key Points

- Headings, lists, tables, code blocks — all supported
- Brand colours, fonts, and layout applied automatically
- Cover pages, headers, and footers included

> Blockquotes render with an accent bar and callout background.

## Next Steps

1. Select a brand profile from the dropdown
2. Write or paste your markdown
3. Click Preview to see the branded PDF
4. Export as PDF or DOCX
`;

const TYPES = Object.entries(TYPE_LABELS);

// The title from the frontmatter, and the body's word count.
function describe(markdown: string): { title: string | null; words: number } {
  const m = markdown.match(/^---\n([\s\S]*?)\n---\n?/);
  let title: string | null = null;
  if (m) {
    try {
      const fm = parseYaml(m[1]);
      if (fm && typeof fm.title === "string") title = fm.title;
    } catch {
      // Half-typed frontmatter: no title until it parses.
    }
  }
  const body = m ? markdown.slice(m[0].length) : markdown;
  const words = body.split(/\s+/).filter((w) => /\w/.test(w)).length;
  return { title, words };
}

export default function EditorPage() {
  const [markdown, setMarkdown] = useState(DEFAULT_MD);
  const [clients, setClients] = useState<Client[]>([]);
  const [selectedClient, setSelectedClient] = useState<string>("");
  const [docType, setDocType] = useState("general");
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [renderMs, setRenderMs] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const { title, words } = useMemo(() => describe(markdown), [markdown]);
  const lines = markdown.split("\n").length;
  const client = clients.find((c) => c.slug === selectedClient);
  const typeLabel = TYPES.find(([v]) => v === docType)?.[1] ?? docType;

  useEffect(() => {
    getClients()
      .then(setClients)
      .catch(() => {});
  }, []);

  const handlePreview = useCallback(async () => {
    setLoading(true);
    setError(null);
    const started = performance.now();
    try {
      const blob = await previewDocument({
        client: selectedClient || undefined,
        document_type: docType,
        format: "pdf",
        content: markdown,
      });
      const url = URL.createObjectURL(blob);
      setPreviewUrl((prev) => {
        if (prev) URL.revokeObjectURL(prev);
        return url;
      });
      setRenderMs(Math.round(performance.now() - started));
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [markdown, selectedClient, docType]);

  const handleDownload = useCallback(
    async (format: "pdf" | "docx") => {
      setLoading(true);
      setError(null);
      try {
        const blob = await previewDocument({
          client: selectedClient || undefined,
          document_type: docType,
          format,
          content: markdown,
        });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `document.${format}`;
        a.click();
        URL.revokeObjectURL(url);
      } catch (e: unknown) {
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setLoading(false);
      }
    },
    [markdown, selectedClient, docType]
  );

  return (
    <>
      <Nav />
      <main className="ts-wrap flex flex-col gap-4 pb-10">
        <div className="flex flex-wrap items-end justify-between gap-x-8 gap-y-4 pt-2 pb-4">
          <div className="flex min-w-0 flex-col gap-5">
            <nav aria-label="Breadcrumb">
              <ol className="kit-breadcrumbs">
                <li>Editor</li>
                <li>{client ? client.name : "No brand"}</li>
                <li aria-current="page">{typeLabel}</li>
              </ol>
            </nav>
            <h1 className="ts-display" data-size="md">{title || "Untitled"}</h1>
          </div>
          <span className="kit-status" data-tone={renderMs !== null ? "good" : undefined} aria-live="polite">
            {loading ? "Rendering" : renderMs !== null ? `Rendered in ${renderMs} ms` : "Not rendered yet"}
          </span>
        </div>

        <div className="ts-editor-bar">
          <label className="ts-mono text-muted">
            Brand
            <select
              className="kit-select"
              data-size="sm"
              value={selectedClient}
              onChange={(e) => setSelectedClient(e.target.value)}
            >
              <option value="">Default (no brand)</option>
              {clients.map((c) => (
                <option key={c.slug} value={c.slug}>
                  {c.name}
                  {c.brand_version ? ` · v${c.brand_version}` : ""}
                </option>
              ))}
            </select>
          </label>

          <label className="ts-mono text-muted">
            Type
            <select className="kit-select" data-size="sm" value={docType} onChange={(e) => setDocType(e.target.value)}>
              {TYPES.map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>

          <div className="flex-1" />

          <div className="flex items-center gap-3">
            <span className="ts-mono text-muted" id="download-label">Download</span>
            <div className="kit-button-group" role="group" aria-labelledby="download-label">
              <button className="kit-button" data-size="sm" onClick={() => handleDownload("pdf")} disabled={loading}>
                PDF
              </button>
              <button className="kit-button" data-size="sm" onClick={() => handleDownload("docx")} disabled={loading}>
                DOCX
              </button>
            </div>
          </div>

          <button
            className="kit-button rounded-full!"
            data-variant="primary"
            onClick={handlePreview}
            disabled={loading}
            aria-busy={loading}
          >
            {loading ? "Rendering" : "Preview"}
          </button>
        </div>

        {error && (
          <div className="kit-alert" data-tone="bad" role="alert">
            <div>
              <p className="kit-alert-title">The render didn&apos;t work</p>
              <p>{error}</p>
            </div>
          </div>
        )}

        <div className="grid gap-4 lg:grid-cols-2">
          <section className="kit-card ts-editor-panel" aria-labelledby="md-label">
            <header className="kit-card-header">
              <h2 id="md-label" className="ts-mono m-0 font-medium">Markdown</h2>
              <span className="ts-mono text-muted">
                {words} words · {lines} lines
              </span>
            </header>
            <div className="kit-card-body">
              <textarea
                value={markdown}
                onChange={(e) => setMarkdown(e.target.value)}
                className="ts-editor-text"
                spellCheck={false}
                aria-labelledby="md-label"
                placeholder="Write your markdown here..."
              />
            </div>
          </section>

          <section className="kit-card ts-editor-panel" data-tone="inverse" aria-labelledby="pv-label">
            <header className="kit-card-header">
              <h2 id="pv-label" className="ts-mono m-0 font-medium">Preview</h2>
              {previewUrl && <span className="kit-badge" data-tone="accent">PDF</span>}
            </header>
            <div className="kit-card-body">
              {previewUrl ? (
                <iframe src={previewUrl} className="ts-preview-frame" title="PDF preview" />
              ) : (
                <div className="flex flex-1 items-center justify-center">
                  <div className="kit-empty">
                    <p className="kit-empty-title">Nothing rendered yet</p>
                    <p>Press Preview to see the branded PDF here.</p>
                  </div>
                </div>
              )}
            </div>
          </section>
        </div>
      </main>
    </>
  );
}
