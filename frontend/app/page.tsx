import Link from "next/link";

export default function Home() {
  return (
    <div className="flex h-full flex-col">
      <nav className="flex items-center justify-between border-b border-border px-6 py-3">
        <div className="flex items-center gap-2">
          <span className="text-lg font-semibold tracking-tight">Typeset</span>
          <span className="text-xs text-muted">v0.1</span>
        </div>
        <div className="flex items-center gap-4">
          <Link
            href="/editor"
            className="rounded-md bg-accent px-4 py-1.5 text-sm font-medium text-white hover:bg-accent-hover transition-colors"
          >
            New Document
          </Link>
          <Link
            href="/profiles"
            className="text-sm text-muted hover:text-foreground transition-colors"
          >
            Brand Profiles
          </Link>
          <Link
            href="/history"
            className="text-sm text-muted hover:text-foreground transition-colors"
          >
            History
          </Link>
        </div>
      </nav>

      <main className="flex flex-1 items-center justify-center">
        <div className="max-w-xl text-center">
          <h1 className="text-4xl font-bold tracking-tight">
            Markdown to branded docs
          </h1>
          <p className="mt-3 text-lg text-muted">
            Write in Markdown. Apply your client&apos;s brand. Export as PDF or
            DOCX in under a second.
          </p>
          <div className="mt-8 flex items-center justify-center gap-3">
            <Link
              href="/editor"
              className="rounded-md bg-accent px-6 py-2.5 text-sm font-medium text-white hover:bg-accent-hover transition-colors"
            >
              Open Editor
            </Link>
            <Link
              href="/profiles"
              className="rounded-md border border-border px-6 py-2.5 text-sm font-medium text-foreground hover:bg-surface transition-colors"
            >
              Manage Brands
            </Link>
          </div>
        </div>
      </main>
    </div>
  );
}
