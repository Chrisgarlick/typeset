"use client";

import { useState, useEffect, useCallback } from "react";
import { Nav } from "@/components/nav";
import { previewDocument, getClients, type Client } from "@/lib/api";

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

export default function EditorPage() {
  const [markdown, setMarkdown] = useState(DEFAULT_MD);
  const [clients, setClients] = useState<Client[]>([]);
  const [selectedClient, setSelectedClient] = useState<string>("");
  const [docType, setDocType] = useState("general");
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getClients()
      .then(setClients)
      .catch(() => {});
  }, []);

  const handlePreview = useCallback(async () => {
    setLoading(true);
    setError(null);
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
    } catch (e: any) {
      setError(e.message);
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
      } catch (e: any) {
        setError(e.message);
      } finally {
        setLoading(false);
      }
    },
    [markdown, selectedClient, docType]
  );

  return (
    <div className="flex h-full flex-col">
      <Nav />

      {/* Toolbar */}
      <div className="flex flex-wrap items-center gap-3 border-b border-border bg-surface px-4 py-2.5 shrink-0">
        <label className="flex items-center gap-2 text-sm text-muted">
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

        <label className="flex items-center gap-2 text-sm text-muted">
          Type
          <select className="kit-select" data-size="sm" value={docType} onChange={(e) => setDocType(e.target.value)}>
            <option value="general">General</option>
            <option value="proposal">Proposal</option>
            <option value="report">Report</option>
            <option value="brief">Brief</option>
            <option value="sop">SOP</option>
            <option value="invoice">Invoice</option>
          </select>
        </label>

        <div className="flex-1" />

        <button className="kit-button" data-variant="primary" data-size="sm" onClick={handlePreview} disabled={loading} aria-busy={loading}>
          {loading && (
            <span className="kit-spinner" data-size="sm" role="status">
              <span className="kit-vh">Rendering</span>
            </span>
          )}
          {loading ? "Rendering" : "Preview"}
        </button>

        <div className="kit-button-group" role="group" aria-label="Download">
          <button className="kit-button" data-size="sm" onClick={() => handleDownload("pdf")} disabled={loading}>
            PDF
          </button>
          <button className="kit-button" data-size="sm" onClick={() => handleDownload("docx")} disabled={loading}>
            DOCX
          </button>
        </div>
      </div>

      {error && (
        <div className="kit-alert m-3 shrink-0" data-tone="bad" role="alert">
          <div>
            <p className="kit-alert-title">The render didn&apos;t work</p>
            <p>{error}</p>
          </div>
        </div>
      )}

      {/* Editor + Preview */}
      <div className="flex flex-1 min-h-0">
        {/* Markdown editor */}
        <div className="flex flex-1 flex-col border-r border-border">
          <div className="border-b border-border px-4 py-2 text-xs text-muted">Markdown</div>
          <textarea
            value={markdown}
            onChange={(e) => setMarkdown(e.target.value)}
            className="editor-textarea flex-1 w-full bg-transparent p-4 text-foreground outline-none"
            spellCheck={false}
            aria-label="Markdown"
            placeholder="Write your markdown here..."
          />
        </div>

        {/* PDF preview */}
        <div className="flex flex-1 flex-col bg-surface-2">
          <div className="border-b border-border px-4 py-2 text-xs text-muted">Preview</div>
          {previewUrl ? (
            <iframe src={previewUrl} className="h-full w-full flex-1" title="PDF preview" />
          ) : (
            <div className="flex flex-1 items-center justify-center p-6">
              <div className="kit-empty">
                <p className="kit-empty-title">Nothing rendered yet</p>
                <p>Press Preview to see the branded PDF here.</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
