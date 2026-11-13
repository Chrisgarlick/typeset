import Link from "next/link";
import logo from "@/brand/assets/typeset-logo-on-dark.svg";

// kit has no footer: this one is Typeset's own (app/typeset.css).
export function Footer() {
  return (
    <footer className="ts-footer">
      <div className="ts-footer-row">
        <Link href="/">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img src={logo.src} alt="Typeset" width={93} height={28} />
        </Link>
        <nav aria-label="Footer">
          <Link href="/editor">Editor</Link>
          <Link href="/history">History</Link>
          <Link href="/#api">API</Link>
          <a href="https://chrisgarlick.com">chrisgarlick.com ↗</a>
        </nav>
      </div>
      <div className="ts-footer-meta ts-mono">
        <span>Built and run by Chris Garlick. Rust, Typst and kit.</span>
        <span>© {new Date().getFullYear()}</span>
      </div>
    </footer>
  );
}
