"use client";

import { useState, useEffect, useCallback } from "react";
import { Nav } from "@/components/nav";
import { previewDocument, getClients, type ClientProfile } from "@/lib/api";

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
  const [clients, setClients] = useState<ClientProfile[]>([]);
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
      <div className="flex items-center gap-3 border-b border-border px-4 py-2 shrink-0">
        <select
          value={selectedClient}
          onChange={(e) => setSelectedClient(e.target.value)}
          className="rounded-md border border-border bg-surface px-3 py-1.5 text-sm text-foreground"
        >
          <option value="">Default (no brand)</option>
          {clients.map((c) => (
            <option key={c.slug} value={c.slug}>
              {c.name}
            </option>
          ))}
        </select>

        <select
          value={docType}
          onChange={(e) => setDocType(e.target.value)}
          className="rounded-md border border-border bg-surface px-3 py-1.5 text-sm text-foreground"
        >
          <option value="general">General</option>
          <option value="proposal">Proposal</option>
          <option value="report">Report</option>
          <option value="brief">Brief</option>
          <option value="sop">SOP</option>
          <option value="invoice">Invoice</option>
        </select>

        <div className="flex-1" />

        {error && (
          <span className="text-xs text-danger">{error}</span>
        )}

        <button
          onClick={handlePreview}
          disabled={loading}
          className="rounded-md bg-accent px-4 py-1.5 text-sm font-medium text-white hover:bg-accent-hover transition-colors disabled:opacity-50"
        >
          {loading ? "Rendering..." : "Preview"}
        </button>

        <button
          onClick={() => handleDownload("pdf")}
          disabled={loading}
          className="rounded-md border border-border px-4 py-1.5 text-sm font-medium text-foreground hover:bg-surface transition-colors disabled:opacity-50"
        >
          PDF
        </button>

        <button
          onClick={() => handleDownload("docx")}
          disabled={loading}
          className="rounded-md border border-border px-4 py-1.5 text-sm font-medium text-foreground hover:bg-surface transition-colors disabled:opacity-50"
        >
          DOCX
        </button>
      </div>

      {/* Editor + Preview */}
      <div className="flex flex-1 min-h-0">
        {/* Markdown editor */}
        <div className="flex-1 border-r border-border">
          <textarea
            value={markdown}
            onChange={(e) => setMarkdown(e.target.value)}
            className="editor-textarea h-full w-full bg-transparent p-4 text-sm text-foreground outline-none"
            spellCheck={false}
            placeholder="Write your markdown here..."
          />
        </div>

        {/* PDF preview */}
        <div className="flex-1 flex items-center justify-center bg-surface">
          {previewUrl ? (
            <iframe
              src={previewUrl}
              className="h-full w-full"
              title="PDF Preview"
            />
          ) : (
            <div className="text-center text-muted">
              <p className="text-sm">Click Preview to render your document</p>
              <p className="mt-1 text-xs">
                The branded PDF will appear here
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
