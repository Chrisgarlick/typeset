"use client";

import { Suspense, useState, useEffect, useRef } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import YAML from "yaml";
import { Nav } from "@/components/nav";
import { saveClient, getClient } from "@/lib/api";

interface FormData {
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
  cover_enabled: boolean;
  cover_template: string;
  header_enabled: boolean;
  footer_enabled: boolean;
}

const DEFAULTS: FormData = {
  slug: "",
  name: "",
  colour_primary: "#000000",
  colour_secondary: "#333333",
  colour_accent: "#0066CC",
  colour_text: "#1A1A1A",
  colour_background: "#FFFFFF",
  colour_table_header: "#F5F5F5",
  colour_table_border: "#E0E0E0",
  colour_callout_bg: "#F8F9FA",
  font_heading: "Helvetica",
  font_body: "Helvetica",
  font_mono: "Courier",
  font_size_base: 11,
  font_size_h1: 24,
  font_size_h2: 18,
  font_size_h3: 14,
  line_height: 1.5,
  page_size: "A4",
  margin_top: 25.4,
  margin_bottom: 25.4,
  margin_left: 25.4,
  margin_right: 25.4,
  cover_enabled: false,
  cover_template: "minimal",
  header_enabled: true,
  footer_enabled: true,
};

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="space-y-3">
      <h3 className="text-sm font-semibold text-muted uppercase tracking-wide">{title}</h3>
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">{children}</div>
    </div>
  );
}

function ColourField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <label className="flex items-center gap-2">
      <input
        type="color"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="h-8 w-8 cursor-pointer rounded border border-border bg-transparent"
      />
      <div>
        <p className="text-xs text-muted">{label}</p>
        <p className="text-xs font-mono">{value}</p>
      </div>
    </label>
  );
}

function TextField({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
}) {
  return (
    <label className="space-y-1">
      <span className="text-xs text-muted">{label}</span>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-full rounded-md border border-border bg-surface px-3 py-1.5 text-sm text-foreground outline-none focus:border-accent"
      />
    </label>
  );
}

function NumberField({
  label,
  value,
  onChange,
  step,
}: {
  label: string;
  value: number;
  onChange: (v: number) => void;
  step?: number;
}) {
  return (
    <label className="space-y-1">
      <span className="text-xs text-muted">{label}</span>
      <input
        type="number"
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value) || 0)}
        step={step || 1}
        className="w-full rounded-md border border-border bg-surface px-3 py-1.5 text-sm text-foreground outline-none focus:border-accent"
      />
    </label>
  );
}

export default function ProfileFormPage() {
  return (
    <Suspense>
      <ProfileForm />
    </Suspense>
  );
}

function ProfileForm() {
  const router = useRouter();
  const params = useSearchParams();
  const editSlug = params.get("edit");

  const [form, setForm] = useState<FormData>(DEFAULTS);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (editSlug) {
      getClient(editSlug).then((p) => {
        setForm({
          slug: p.slug,
          name: p.name,
          colour_primary: p.colour_primary,
          colour_secondary: p.colour_secondary,
          colour_accent: p.colour_accent,
          colour_text: p.colour_text,
          colour_background: p.colour_background,
          colour_table_header: p.colour_table_header,
          colour_table_border: p.colour_table_border,
          colour_callout_bg: p.colour_callout_bg,
          font_heading: p.font_heading,
          font_body: p.font_body,
          font_mono: p.font_mono,
          font_size_base: p.font_size_base,
          font_size_h1: p.font_size_h1,
          font_size_h2: p.font_size_h2,
          font_size_h3: p.font_size_h3,
          line_height: p.line_height,
          page_size: p.page_size,
          margin_top: p.margin_top,
          margin_bottom: p.margin_bottom,
          margin_left: p.margin_left,
          margin_right: p.margin_right,
          cover_enabled: p.cover_enabled,
          cover_template: p.cover_template,
          header_enabled: p.header_enabled,
          footer_enabled: p.footer_enabled,
        });
      });
    }
  }, [editSlug]);

  const set = <K extends keyof FormData>(key: K, val: FormData[K]) =>
    setForm((f) => ({ ...f, [key]: val }));

  const fileRef = useRef<HTMLInputElement>(null);

  const handleSave = async () => {
    if (!form.slug || !form.name) {
      setError("Slug and name are required");
      return;
    }
    setSaving(true);
    setError(null);
    try {
      await saveClient(form);
      router.push("/profiles");
    } catch (e: any) {
      setError(e.message);
    } finally {
      setSaving(false);
    }
  };

  const handleImportYaml = (file: File) => {
    const reader = new FileReader();
    reader.onload = (e) => {
      try {
        const parsed = YAML.parse(e.target?.result as string);
        setForm((prev) => ({ ...prev, ...parsed }));
        setError(null);
      } catch {
        setError("Invalid YAML file");
      }
    };
    reader.readAsText(file);
  };

  const handleExportYaml = () => {
    const yml = YAML.stringify(form);
    const blob = new Blob([yml], { type: "text/yaml" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${form.slug || "profile"}.yaml`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="flex h-full flex-col">
      <Nav />
      <div className="flex-1 overflow-auto p-6">
        <div className="mx-auto max-w-2xl space-y-8">
          <div className="flex items-start justify-between">
            <div>
              <h1 className="text-2xl font-bold">
                {editSlug ? "Edit Profile" : "New Brand Profile"}
              </h1>
              <p className="mt-1 text-sm text-muted">
                Configure how documents look for this client
              </p>
            </div>
            <div className="flex items-center gap-2">
              <input
                ref={fileRef}
                type="file"
                accept=".yaml,.yml"
                className="hidden"
                onChange={(e) => {
                  const f = e.target.files?.[0];
                  if (f) handleImportYaml(f);
                }}
              />
              <button
                onClick={() => fileRef.current?.click()}
                className="rounded-md border border-border px-3 py-1.5 text-xs text-muted hover:text-foreground transition-colors"
              >
                Import YAML
              </button>
              <button
                onClick={handleExportYaml}
                className="rounded-md border border-border px-3 py-1.5 text-xs text-muted hover:text-foreground transition-colors"
              >
                Export YAML
              </button>
            </div>
          </div>

          {/* Identity */}
          <Section title="Identity">
            <TextField
              label="Slug (URL-safe ID)"
              value={form.slug}
              onChange={(v) => set("slug", v.toLowerCase().replace(/[^a-z0-9-]/g, ""))}
              placeholder="acme-corp"
            />
            <TextField
              label="Display Name"
              value={form.name}
              onChange={(v) => set("name", v)}
              placeholder="Acme Corporation"
            />
          </Section>

          {/* Colours */}
          <Section title="Colours">
            <ColourField label="Primary" value={form.colour_primary} onChange={(v) => set("colour_primary", v)} />
            <ColourField label="Secondary" value={form.colour_secondary} onChange={(v) => set("colour_secondary", v)} />
            <ColourField label="Accent" value={form.colour_accent} onChange={(v) => set("colour_accent", v)} />
            <ColourField label="Text" value={form.colour_text} onChange={(v) => set("colour_text", v)} />
            <ColourField label="Background" value={form.colour_background} onChange={(v) => set("colour_background", v)} />
            <ColourField label="Table Header" value={form.colour_table_header} onChange={(v) => set("colour_table_header", v)} />
            <ColourField label="Table Border" value={form.colour_table_border} onChange={(v) => set("colour_table_border", v)} />
            <ColourField label="Callout BG" value={form.colour_callout_bg} onChange={(v) => set("colour_callout_bg", v)} />
          </Section>

          {/* Typography */}
          <Section title="Typography">
            <TextField label="Heading Font" value={form.font_heading} onChange={(v) => set("font_heading", v)} />
            <TextField label="Body Font" value={form.font_body} onChange={(v) => set("font_body", v)} />
            <TextField label="Mono Font" value={form.font_mono} onChange={(v) => set("font_mono", v)} />
            <NumberField label="Base Size (pt)" value={form.font_size_base} onChange={(v) => set("font_size_base", v)} step={0.5} />
            <NumberField label="H1 Size (pt)" value={form.font_size_h1} onChange={(v) => set("font_size_h1", v)} />
            <NumberField label="H2 Size (pt)" value={form.font_size_h2} onChange={(v) => set("font_size_h2", v)} />
            <NumberField label="H3 Size (pt)" value={form.font_size_h3} onChange={(v) => set("font_size_h3", v)} />
            <NumberField label="Line Height" value={form.line_height} onChange={(v) => set("line_height", v)} step={0.1} />
          </Section>

          {/* Layout */}
          <Section title="Layout">
            <label className="space-y-1">
              <span className="text-xs text-muted">Page Size</span>
              <select
                value={form.page_size}
                onChange={(e) => set("page_size", e.target.value)}
                className="w-full rounded-md border border-border bg-surface px-3 py-1.5 text-sm text-foreground"
              >
                <option value="A4">A4</option>
                <option value="Letter">Letter</option>
                <option value="Legal">Legal</option>
              </select>
            </label>
            <NumberField label="Top Margin (mm)" value={form.margin_top} onChange={(v) => set("margin_top", v)} step={0.1} />
            <NumberField label="Bottom Margin (mm)" value={form.margin_bottom} onChange={(v) => set("margin_bottom", v)} step={0.1} />
            <NumberField label="Left Margin (mm)" value={form.margin_left} onChange={(v) => set("margin_left", v)} step={0.1} />
            <NumberField label="Right Margin (mm)" value={form.margin_right} onChange={(v) => set("margin_right", v)} step={0.1} />
          </Section>

          {/* Options */}
          <Section title="Options">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={form.cover_enabled}
                onChange={(e) => set("cover_enabled", e.target.checked)}
                className="rounded border-border"
              />
              <span className="text-sm">Cover page</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={form.header_enabled}
                onChange={(e) => set("header_enabled", e.target.checked)}
                className="rounded border-border"
              />
              <span className="text-sm">Header</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={form.footer_enabled}
                onChange={(e) => set("footer_enabled", e.target.checked)}
                className="rounded border-border"
              />
              <span className="text-sm">Footer with page numbers</span>
            </label>
          </Section>

          {/* Preview strip */}
          <div>
            <h3 className="text-sm font-semibold text-muted uppercase tracking-wide mb-3">Preview</h3>
            <div
              className="rounded-lg border p-6 space-y-2"
              style={{
                background: form.colour_background,
                borderColor: form.colour_table_border,
              }}
            >
              <h2
                style={{
                  color: form.colour_primary,
                  fontFamily: form.font_heading,
                  fontSize: `${form.font_size_h1}px`,
                  fontWeight: 700,
                }}
              >
                Heading One
              </h2>
              <div style={{ borderBottom: `2px solid ${form.colour_accent}`, width: 80 }} />
              <h3
                style={{
                  color: form.colour_secondary,
                  fontFamily: form.font_heading,
                  fontSize: `${form.font_size_h2}px`,
                  fontWeight: 600,
                }}
              >
                Heading Two
              </h3>
              <p
                style={{
                  color: form.colour_text,
                  fontFamily: form.font_body,
                  fontSize: `${form.font_size_base}px`,
                  lineHeight: form.line_height,
                }}
              >
                This is body text showing how your brand profile will look in
                rendered documents. The colours, fonts, and sizes shown here
                match your configuration.
              </p>
              <div
                className="rounded p-3 border-l-4"
                style={{
                  background: form.colour_callout_bg,
                  borderColor: form.colour_accent,
                }}
              >
                <p
                  style={{
                    color: form.colour_text,
                    fontFamily: form.font_body,
                    fontSize: `${form.font_size_base}px`,
                  }}
                >
                  This is a blockquote callout.
                </p>
              </div>
            </div>
          </div>

          {/* Actions */}
          {error && (
            <p className="text-sm text-danger">{error}</p>
          )}
          <div className="flex items-center gap-3 pb-8">
            <button
              onClick={handleSave}
              disabled={saving}
              className="rounded-md bg-accent px-6 py-2.5 text-sm font-medium text-white hover:bg-accent-hover transition-colors disabled:opacity-50"
            >
              {saving ? "Saving..." : editSlug ? "Update Profile" : "Create Profile"}
            </button>
            <button
              onClick={() => router.push("/profiles")}
              className="rounded-md border border-border px-6 py-2.5 text-sm font-medium text-foreground hover:bg-surface transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
