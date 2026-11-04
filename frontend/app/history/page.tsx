"use client";

import { useState, useEffect } from "react";
import { Nav } from "@/components/nav";
import { getHistory, type RenderHistoryItem } from "@/lib/api";

const when = (iso: string) =>
  new Date(iso).toLocaleDateString("en-GB", { day: "2-digit", month: "short", year: "numeric", hour: "2-digit", minute: "2-digit" });

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
      <main className="flex-1 overflow-auto">
        <div className="kit-container py-8">
          <div className="mb-6 flex items-center gap-3">
            <h1 className="text-3xl font-bold">Render history</h1>
            {!loading && <span className="kit-badge">{items.length}</span>}
          </div>

          {loading ? (
            <div className="flex items-center gap-3 text-sm text-muted">
              <span className="kit-spinner" data-size="sm" role="status">
                <span className="kit-vh">Loading</span>
              </span>
              Loading renders
            </div>
          ) : items.length === 0 ? (
            <div className="kit-empty">
              <p className="kit-empty-title">No renders yet</p>
              <p>Documents rendered through the API or the editor&apos;s downloads appear here.</p>
            </div>
          ) : (
            <div className="kit-table-wrap" role="region" aria-label="Render history" tabIndex={0}>
              <table className="kit-table">
                <thead>
                  <tr>
                    <th scope="col">Date</th>
                    <th scope="col">Client</th>
                    <th scope="col">Type</th>
                    <th scope="col">Format</th>
                    <th scope="col" data-numeric>Time</th>
                  </tr>
                </thead>
                <tbody>
                  {items.map((item) => (
                    <tr key={item.id}>
                      <td className="text-muted">{when(item.created_at)}</td>
                      <td>
                        {item.client_slug ? (
                          <span className="kit-badge" data-tone="accent">{item.client_slug}</span>
                        ) : (
                          <span className="text-muted">Default</span>
                        )}
                      </td>
                      <td className="capitalize">{item.document_type}</td>
                      <td>
                        <span className="kit-badge" data-tone="info">{item.format.toUpperCase()}</span>
                      </td>
                      <td className="text-muted" data-numeric>{item.render_ms ? `${item.render_ms} ms` : "—"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
