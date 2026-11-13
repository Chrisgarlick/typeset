"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
import logo from "@/brand/assets/typeset-logo.svg";
import { getHealth } from "@/lib/api";

// kit's top bar. Typeset draws its links in a white pill (app/typeset.css).
export function Nav() {
  const path = usePathname();
  const current = (href: string) => (path.startsWith(href) ? "page" : undefined);
  const [up, setUp] = useState<boolean | null>(null);

  useEffect(() => {
    getHealth().then(setUp);
  }, []);

  return (
    <header className="kit-topbar">
      <Link className="kit-topbar-brand" href="/">
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img src={logo.src} alt="Typeset" width={113} height={34} />
      </Link>
      <nav className="kit-topbar-nav" aria-label="Main">
        <Link href="/editor" aria-current={current("/editor")}>Editor</Link>
        <Link href="/history" aria-current={current("/history")}>History</Link>
        <Link href="/#api">API</Link>
        <a href="https://clients.kritano.com">
          Brands <span aria-hidden="true">↗</span>
        </a>
      </nav>
      <div className="kit-topbar-actions">
        {up !== null && (
          <span className="kit-status" data-tone={up ? "good" : "bad"}>
            {up ? "API up" : "API down"}
          </span>
        )}
        <Link className="kit-button" data-variant="primary" href="/editor">
          New document
        </Link>
      </div>
    </header>
  );
}
