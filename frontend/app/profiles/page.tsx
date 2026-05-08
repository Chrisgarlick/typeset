"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { Nav } from "@/components/nav";
import { getClients, deleteClient, type ClientProfile } from "@/lib/api";

export default function ProfilesPage() {
  const [profiles, setProfiles] = useState<ClientProfile[]>([]);
  const [loading, setLoading] = useState(true);

  const load = () => {
    getClients()
      .then(setProfiles)
      .catch(() => {})
      .finally(() => setLoading(false));
  };

  useEffect(load, []);

  const handleDelete = async (slug: string) => {
    if (!confirm(`Delete profile "${slug}"?`)) return;
    await deleteClient(slug);
    load();
  };

  return (
    <div className="flex h-full flex-col">
      <Nav />
      <div className="flex-1 overflow-auto p-6">
        <div className="mx-auto max-w-3xl">
          <div className="flex items-center justify-between mb-6">
            <h1 className="text-2xl font-bold">Brand Profiles</h1>
            <Link
              href="/profiles/new"
              className="rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover transition-colors"
            >
              New Profile
            </Link>
          </div>

          {loading ? (
            <p className="text-sm text-muted">Loading...</p>
          ) : profiles.length === 0 ? (
            <div className="rounded-lg border border-border p-8 text-center">
              <p className="text-muted">No brand profiles yet</p>
              <Link
                href="/profiles/new"
                className="mt-3 inline-block text-sm text-accent hover:text-accent-hover"
              >
                Create your first profile
              </Link>
            </div>
          ) : (
            <div className="space-y-3">
              {profiles.map((p) => (
                <div
                  key={p.slug}
                  className="flex items-center justify-between rounded-lg border border-border p-4 hover:bg-surface transition-colors"
                >
                  <div className="flex items-center gap-4">
                    {/* Colour swatches */}
                    <div className="flex gap-1">
                      <div
                        className="h-8 w-8 rounded"
                        style={{ background: p.colour_primary }}
                      />
                      <div
                        className="h-8 w-8 rounded"
                        style={{ background: p.colour_secondary }}
                      />
                      <div
                        className="h-8 w-8 rounded"
                        style={{ background: p.colour_accent }}
                      />
                    </div>
                    <div>
                      <p className="font-medium">{p.name}</p>
                      <p className="text-xs text-muted">
                        {p.slug} &middot; {p.font_heading} / {p.font_body} &middot;{" "}
                        {p.page_size}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Link
                      href={`/profiles/new?edit=${p.slug}`}
                      className="rounded-md border border-border px-3 py-1.5 text-xs text-muted hover:text-foreground transition-colors"
                    >
                      Edit
                    </Link>
                    <button
                      onClick={() => handleDelete(p.slug)}
                      className="rounded-md border border-border px-3 py-1.5 text-xs text-danger hover:bg-surface transition-colors"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
