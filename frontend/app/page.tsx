import Link from "next/link";
import { Nav } from "@/components/nav";

const steps = [
  ["Write", "Markdown with a little frontmatter: title, client, cover, mode."],
  ["Brand", "Each client's published brand in Clients: colours, fonts, logos, print settings."],
  ["Export", "A tagged, accessible PDF or a DOCX, in about a second."],
];

export default function Home() {
  return (
    <div className="flex h-full flex-col">
      <Nav />
      <main className="kit-container flex flex-1 flex-col justify-center py-16">
        <div className="max-w-2xl">
          <span className="kit-badge" data-tone="accent">Typeset</span>
          <h1 className="mt-4 text-5xl font-bold leading-tight">Markdown to branded documents.</h1>
          <p className="mt-4 text-lg text-muted">
            Write in Markdown, pick a client, and export a PDF or DOCX in their brand.
          </p>
          <div className="mt-8 flex flex-wrap gap-3">
            <Link className="kit-button" data-variant="primary" href="/editor">Open the editor</Link>
            <a className="kit-button" href="https://clients.kritano.com">Manage brands in Clients ↗</a>
          </div>
        </div>
        <div className="kit-grid mt-14" data-min="sm" data-cols="3">
          {steps.map(([title, body], i) => (
            <article key={title} className="kit-card">
              <div className="kit-card-body">
                <p className="text-sm text-muted">0{i + 1}</p>
                <h2 className="mt-1 text-lg font-semibold">{title}</h2>
                <p className="mt-2 text-sm text-muted">{body}</p>
              </div>
            </article>
          ))}
        </div>
      </main>
    </div>
  );
}
