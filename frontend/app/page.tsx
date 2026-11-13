import Link from "next/link";
import { Nav } from "@/components/nav";
import { Footer } from "@/components/footer";
import { DocPage } from "@/components/doc-page";
import { CopyButton } from "@/components/copy-button";

const strip = ["Tagged PDF/UA-1", "DOCX", "Cover pages", "Charts & tables", "Diagrams", "Brand fonts", "One API call"];

const stats = [
  ["~1 s", "A typical render, from request to file. Typst under the hood, written in Rust."],
  ["PDF/UA", "Tagged PDFs: real headings, reading order and alt text, so screen readers can follow them."],
  ["6", "Document types: general, proposal, report, brief, SOP and invoice."],
  ["2", "Formats from one source: PDF to send, DOCX for the client to edit."],
];

const blocks = [
  ["Cover pages", "Cover", "Generated from the frontmatter, or your own designed page from Design."],
  ["Charts", "Chart", "Bar, line and donut charts from a table of numbers, in the brand's series colours."],
  ["Tables", "Table", "Markdown tables with header rows that repeat across pages."],
  ["Diagrams", "Fletcher", "Flowcharts and sequence diagrams from a fletcher code block."],
  ["Checklists & trees", "List", "Tick lists for SOPs and file trees for technical handovers."],
  ["Callouts", "Note", "Blockquotes with an accent bar, and notes, tips and warnings."],
];

const curl = `curl -X POST https://typeset.chrisgarlick.com/api/render \\
  -H "Authorization: Bearer $TYPESET_TOKEN" \\
  -H "Content-Type: application/json" \\
  -d '{
    "client": "kritano",
    "document_type": "proposal",
    "format": "both",
    "content": "# Executive summary ..."
  }'`;

export default function Home() {
  return (
    <>
      <Nav />
      <main>
        <section className="ts-wrap flex items-center gap-12 pt-10 pb-24" aria-labelledby="hero-title">
          <div className="flex min-w-0 flex-1 flex-col items-start gap-8">
            <span className="kit-badge" data-tone="accent">
              <span className="kit-dot" />
              Markdown → PDF · DOCX
            </span>
            <h1 id="hero-title" className="ts-display" data-size="hero">
              Markdown in.
              <br />
              <span className="ts-mark">Branded</span>
              <br />
              PDF out.
            </h1>
            <p className="ts-lede">
              Write in Markdown, pick a client, and Typeset sets it in their brand: their colours, fonts, logos and
              cover page. A tagged, accessible PDF or a DOCX in about a second.
            </p>
            <div className="flex flex-wrap gap-4">
              <Link className="kit-button" data-variant="primary" data-size="lg" data-arrow href="/editor">
                Open the editor
              </Link>
              <a className="kit-button" data-size="lg" href="#api">
                Read the API
              </a>
            </div>
          </div>
          <div className="ts-fan">
            <DocPage client="Acme Corp" kind="Report" title={"Quarterly\nreview"} site="acme.example" pages="01 / 12" colour="#C2410C" soft="#FBE9DF" />
            <DocPage client="Chris Garlick" kind="Scope" title={"Laravel\nrebuild"} site="chrisgarlick.com" pages="01 / 04" colour="#151513" soft="#F2EEE6" />
            <DocPage client="Kritano" kind="Proposal" title={"Site audit\nand fixes"} site="kritano.com" pages="01 / 06" colour="#1F5C45" soft="#DDE7DF" />
            <span className="ts-fan-note">Rendered in 0.9 s</span>
          </div>
        </section>

        <ul className="ts-strip" aria-label="What Typeset makes">
          {strip.map((s) => (
            <li key={s}>{s}</li>
          ))}
        </ul>

        <section className="ts-wrap flex flex-col items-start gap-12 pt-24 pb-20" aria-labelledby="how-title">
          <span className="kit-badge" data-tone="accent">How it works</span>
          <h2 id="how-title" className="ts-display" data-size="lg">
            Three steps.
            <br />
            No design tool.
          </h2>
          <div className="grid w-full gap-4 lg:grid-cols-3">
            <article className="kit-card min-h-[480px]">
              <header className="kit-card-header">
                <h3 className="ts-card-title">Write</h3>
                <span className="ts-card-num" aria-hidden="true">01</span>
              </header>
              <div className="kit-card-body">
                <figure className="kit-code flex-1">
                  <pre tabIndex={0} aria-label="Markdown example">
                    <code>
                      <span className="ts-dim">---</span>
                      {"\n"}
                      <span className="ts-hl">title: Site audit and fixes</span>
                      {"\n"}
                      <span className="ts-hl">client: kritano</span>
                      {"\n"}
                      <span className="ts-dim">---</span>
                      {"\n# Executive summary\n"}
                      <span className="ts-dim">Your site scored **72**...</span>
                    </code>
                  </pre>
                </figure>
                <p>Markdown with a little frontmatter: title, client, cover, mode. Or JSON, from your own app.</p>
              </div>
            </article>
            <article className="kit-card min-h-[480px]">
              <header className="kit-card-header">
                <h3 className="ts-card-title">Brand</h3>
                <span className="ts-card-num" aria-hidden="true">02</span>
              </header>
              <div className="kit-card-body">
                <div className="flex flex-1 flex-col justify-center gap-5 rounded-2xl bg-background p-5">
                  <div className="flex gap-3" aria-hidden="true">
                    {["#1F5C45", "var(--kit-fg)", "var(--kit-accent-soft)", "#C2410C"].map((c) => (
                      <span key={c} className="size-11 rounded-full" style={{ background: c }} />
                    ))}
                  </div>
                  <p className="m-0 text-[15px] font-medium text-foreground">Aa Barlow Condensed · Inter · JetBrains Mono</p>
                  <div className="flex flex-wrap gap-3">
                    <span className="kit-badge" data-tone="accent">kritano · v3</span>
                    <span className="kit-badge" data-tone="accent" style={{ background: "var(--kit-surface)" }}>acme · v1</span>
                  </div>
                </div>
                <p>Each client&apos;s published brand from Clients: colours, fonts, logos and print settings, pinned by version.</p>
              </div>
            </article>
            <article className="kit-card min-h-[480px]" data-tone="inverse">
              <header className="kit-card-header">
                <h3 className="ts-card-title">Export</h3>
                <span className="ts-card-num" aria-hidden="true">03</span>
              </header>
              <div className="kit-card-body">
                <div className="flex flex-1 flex-col justify-end gap-3">
                  <p className="ts-display text-highlight" style={{ "--ts-size": "clamp(96px, 11.7vw, 168px)" } as React.CSSProperties}>0.9 <span className="ts-unit">s</span></p>
                  <p className="ts-mono">From POST to PDF</p>
                </div>
                <p>A tagged, accessible PDF or an editable DOCX, ready to send.</p>
              </div>
            </article>
          </div>
        </section>

        <section className="ts-wrap pb-20" aria-labelledby="out-title">
          <div className="ts-slab flex flex-col gap-12 px-6 pt-16 pb-12 md:px-12">
            <h2 id="out-title" className="ts-display" data-size="lg">What comes out</h2>
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              {stats.map(([value, note]) => (
                <div key={value} className="kit-stat">
                  <p className="kit-stat-value">{value}</p>
                  <p className="kit-stat-note">{note}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section id="api" className="ts-wrap flex scroll-mt-8 flex-wrap items-center gap-12 pt-4 pb-24" aria-labelledby="api-title">
          <div className="flex min-w-[min(100%,24rem)] flex-1 flex-col items-start gap-6">
            <span className="kit-badge" data-tone="accent">The API</span>
            <h2 id="api-title" className="ts-display" data-size="lg" style={{ "--ts-size": "clamp(64px, 10.8vw, 156px)" } as React.CSSProperties}>
              One POST.
              <br />
              One PDF.
            </h2>
            <p className="ts-lede text-lg">
              Send Markdown or JSON from any app, with a bearer key. Typeset answers with links to the PDF and the DOCX,
              and keeps a record in History.
            </p>
          </div>
          <figure className="kit-code w-full max-w-[680px]">
            <figcaption>
              <span>curl</span>
              <CopyButton text={curl} label="Copy the curl example" />
            </figcaption>
            <pre tabIndex={0} id="api-example">
              <code>
                {curl}
                {"\n\n"}
                <span className="ts-hl">{'→ 200  { "pdf_url": "…/r_8f2c.pdf", "render_ms": 912 }'}</span>
              </code>
            </pre>
          </figure>
        </section>

        <section className="ts-wrap flex flex-col items-start gap-12 bg-surface py-24" aria-labelledby="blocks-title">
          <span className="kit-badge" data-tone="accent">Blocks</span>
          <h2 id="blocks-title" className="ts-display" data-size="lg">
            Everything a
            <br />
            document needs
          </h2>
          <div className="grid w-full gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {blocks.map(([title, meta, body]) => (
              <article key={title} className="kit-card" data-tone="canvas">
                <header className="kit-card-header">
                  <h3 className="ts-card-title">{title}</h3>
                  <span className="ts-mono text-muted">{meta}</span>
                </header>
                <div className="kit-card-body">
                  <p>{body}</p>
                </div>
              </article>
            ))}
          </div>
        </section>

        <section className="ts-wrap flex flex-col items-center gap-8 py-32 text-center" aria-labelledby="cta-title">
          <h2 id="cta-title" className="ts-display" data-size="xl">
            Write it once.
            <br />
            Send it branded.
          </h2>
          <div className="flex flex-wrap justify-center gap-4">
            <Link className="kit-button" data-variant="primary" data-size="lg" data-arrow href="/editor">
              Open the editor
            </Link>
            <Link className="kit-button" data-size="lg" href="/history">
              See the history
            </Link>
          </div>
        </section>
      </main>
      <Footer />
    </>
  );
}
