"use client";

import { Suspense, useEffect, useState } from "react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Nav } from "@/components/nav";
import { Footer } from "@/components/footer";
import { getHistory, HISTORY_MAX, TYPE_LABELS, type RenderHistoryItem } from "@/lib/api";

const FORMATS: [string, string][] = [
  ["", "All"],
  ["pdf", "PDF"],
  ["docx", "DOCX"],
  ["both", "Both"],
];

const when = (iso: string) =>
  new Date(iso).toLocaleString("en-GB", { day: "2-digit", month: "short", year: "numeric", hour: "2-digit", minute: "2-digit" }).replace(",", " ·");

const titleOf = (item: RenderHistoryItem) =>
  typeof item.frontmatter?.title === "string" && item.frontmatter.title.trim() ? item.frontmatter.title : "Untitled";

function median(values: number[]): number | null {
  if (values.length === 0) return null;
  const s = [...values].sort((a, b) => a - b);
  const mid = Math.floor(s.length / 2);
  return s.length % 2 ? s[mid] : Math.round((s[mid - 1] + s[mid]) / 2);
}

// Figures from the latest renders the API returns (up to HISTORY_MAX).
function Stats({ items }: { items: RenderHistoryItem[] }) {
  const now = new Date();
  const thisMonth = items.filter((i) => {
    const d = new Date(i.created_at);
    return d.getFullYear() === now.getFullYear() && d.getMonth() === now.getMonth();
  }).length;
  const ms = median(items.flatMap((i) => (i.render_ms ? [i.render_ms] : [])));
  const brands = new Set(items.flatMap((i) => (i.client_slug ? [i.client_slug] : []))).size;
  const week = items.filter((i) => now.getTime() - new Date(i.created_at).getTime() < 7 * 864e5).length;
  const capped = items.length === HISTORY_MAX;

  const tiles: [string, string, string?][] = [
    ["Renders this month", `${thisMonth}${capped && thisMonth === items.length ? "+" : ""}`],
    ms === null ? ["Median render", "—"] : ms < 1000 ? ["Median render", String(ms), "ms"] : ["Median render", (ms / 1000).toFixed(1), "s"],
    ["Brands used", String(brands)],
    ["Last 7 days", `${week}${capped && week === items.length ? "+" : ""}`],
  ];

  return (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
      {tiles.map(([label, value, unit], i) => (
        <div key={label} className="kit-stat" data-tone={i === 0 ? "inverse" : undefined}>
          <p className="kit-stat-label">{label}</p>
          <p className="kit-stat-value">
            {value}
            {unit && <small>{unit}</small>}
          </p>
        </div>
      ))}
    </div>
  );
}

function History() {
  const format = useSearchParams().get("format") ?? "";
  const [all, setAll] = useState<RenderHistoryItem[]>([]);
  // The rows and the filter they were loaded for: loading until they match.
  const [rows, setRows] = useState<{ format: string; items: RenderHistoryItem[] } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const loading = rows?.format !== format && !error;
  const items = rows?.items ?? [];

  // The stats cover every format; the table follows the filter.
  useEffect(() => {
    getHistory(HISTORY_MAX)
      .then(setAll)
      .catch((e: Error) => setError(e.message));
  }, []);

  useEffect(() => {
    getHistory(50, 0, format || undefined)
      .then((items) => setRows({ format, items }))
      .catch((e: Error) => setError(e.message));
  }, [format]);

  return (
    <main className="ts-wrap flex flex-col gap-6 pb-24">
      <div className="flex flex-wrap items-end justify-between gap-6 pt-2 pb-4">
        <div className="flex flex-col gap-5">
          <p className="ts-mono m-0 text-muted">Every render, from the editor and the API</p>
          <h1 className="ts-display" data-size="lg" style={{ "--ts-size": "clamp(72px, 12.2vw, 176px)" } as React.CSSProperties}>
            Render
            <br />
            history
          </h1>
        </div>
        <nav className="kit-segmented" aria-label="Format">
          {FORMATS.map(([value, label]) => (
            <Link key={label} href={value ? `/history?format=${value}` : "/history"} aria-current={format === value ? "page" : undefined}>
              {label}
            </Link>
          ))}
        </nav>
      </div>

      {error && (
        <div className="kit-alert" data-tone="bad" role="alert">
          <div>
            <p className="kit-alert-title">History didn&apos;t load</p>
            <p>{error}</p>
          </div>
        </div>
      )}

      <Stats items={all} />

      {loading ? (
        <div className="flex items-center gap-3 py-8 text-muted">
          <span className="kit-spinner" data-size="sm" role="status">
            <span className="sr-only">Loading</span>
          </span>
          <span className="ts-mono">Loading renders</span>
        </div>
      ) : items.length === 0 ? (
        <div className="kit-card">
          <div className="kit-card-body py-16">
            <div className="kit-empty">
              <p className="kit-empty-title text-foreground!">No renders yet</p>
              <p className="text-muted">Documents rendered through the API or the editor&apos;s downloads appear here.</p>
            </div>
          </div>
        </div>
      ) : (
        <div className="kit-table-wrap" role="region" aria-label="Render history" tabIndex={0}>
          <table className="kit-table">
            <thead>
              <tr>
                <th scope="col">Date</th>
                <th scope="col">Document</th>
                <th scope="col">Client</th>
                <th scope="col">Type</th>
                <th scope="col">Format</th>
                <th scope="col" data-numeric>Time</th>
                <th scope="col"><span className="sr-only">Download</span></th>
              </tr>
            </thead>
            <tbody>
              {items.map((item) => {
                const url = item.pdf_url ?? item.docx_url;
                return (
                  <tr key={item.id}>
                    <td className="ts-cell-date">{when(item.created_at)}</td>
                    <td className="ts-cell-doc">{titleOf(item)}</td>
                    <td>
                      {item.client_slug ? (
                        <span className="kit-badge" data-tone="accent">
                          {item.client_slug}
                          {item.brand_version ? ` · v${item.brand_version}` : ""}
                        </span>
                      ) : (
                        <span className="text-muted">Default</span>
                      )}
                    </td>
                    <td>{TYPE_LABELS[item.document_type] ?? item.document_type}</td>
                    <td>
                      <span className="flex gap-2">
                        {item.pdf_url && <span className="kit-badge" data-tone="neutral">PDF</span>}
                        {item.docx_url && <span className="kit-badge" data-variant="outline">DOCX</span>}
                        {!item.pdf_url && !item.docx_url && <span className="kit-badge" data-variant="outline">{item.format}</span>}
                      </span>
                    </td>
                    <td data-numeric>{item.render_ms ? `${item.render_ms.toLocaleString("en-GB")} ms` : "—"}</td>
                    <td>
                      {url && (
                        <a className="kit-button" data-variant="ghost" data-size="sm" href={url} aria-label={`Download ${titleOf(item)}`}>
                          ↓
                        </a>
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </main>
  );
}

export default function HistoryPage() {
  return (
    <>
      <Nav />
      <Suspense fallback={<main className="ts-wrap pb-24" />}>
        <History />
      </Suspense>
      <Footer />
    </>
  );
}
