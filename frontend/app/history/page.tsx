"use client";

import { useState, useEffect } from "react";
import { Nav } from "@/components/nav";
import { getHistory, type RenderHistoryItem } from "@/lib/api";

export default function HistoryPage() {
  const [items, setItems] = useState<RenderHistoryItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getHistory(50)
      .then(setItems)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  return (
    <div className="flex h-full flex-col">
      <Nav />
      <div className="flex-1 overflow-auto p-6">
        <div className="mx-auto max-w-4xl">
          <h1 className="text-2xl font-bold mb-6">Render History</h1>

          {loading ? (
            <p className="text-sm text-muted">Loading...</p>
          ) : items.length === 0 ? (
            <div className="rounded-lg border border-border p-8 text-center">
              <p className="text-muted">No renders yet</p>
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-border text-left text-xs text-muted uppercase tracking-wide">
                    <th className="pb-2 pr-4">Date</th>
                    <th className="pb-2 pr-4">Client</th>
                    <th className="pb-2 pr-4">Type</th>
                    <th className="pb-2 pr-4">Format</th>
                    <th className="pb-2 pr-4">Time</th>
                    <th className="pb-2">Links</th>
                  </tr>
                </thead>
                <tbody>
                  {items.map((item) => (
                    <tr
                      key={item.id}
                      className="border-b border-border/50 hover:bg-surface transition-colors"
                    >
                      <td className="py-3 pr-4 text-muted">
                        {new Date(item.created_at).toLocaleDateString("en-GB", {
                          day: "2-digit",
                          month: "short",
                          year: "numeric",
                          hour: "2-digit",
                          minute: "2-digit",
                        })}
                      </td>
                      <td className="py-3 pr-4">
                        {item.client_slug || "—"}
                      </td>
                      <td className="py-3 pr-4 capitalize">
                        {item.document_type}
                      </td>
                      <td className="py-3 pr-4 uppercase text-xs font-mono">
                        {item.format}
                      </td>
                      <td className="py-3 pr-4 text-muted">
                        {item.render_ms ? `${item.render_ms}ms` : "—"}
                      </td>
                      <td className="py-3 flex gap-2">
                        {item.pdf_url && (
                          <a
                            href={item.pdf_url}
                            target="_blank"
                            rel="noreferrer"
                            className="text-xs text-accent hover:text-accent-hover"
                          >
                            PDF
                          </a>
                        )}
                        {item.docx_url && (
                          <a
                            href={item.docx_url}
                            target="_blank"
                            rel="noreferrer"
                            className="text-xs text-accent hover:text-accent-hover"
                          >
                            DOCX
                          </a>
                        )}
                        {!item.pdf_url && !item.docx_url && (
                          <span className="text-xs text-muted">—</span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
