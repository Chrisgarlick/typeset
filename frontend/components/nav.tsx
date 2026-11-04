"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

export function Nav() {
  const path = usePathname();
  const current = (href: string) => (path.startsWith(href) ? "page" : undefined);

  return (
    <header className="kit-topbar shrink-0">
      <Link className="kit-topbar-brand" href="/">
        <span className="kit-avatar" data-size="sm" data-tone="3" aria-hidden="true">T</span>
        Typeset
      </Link>
      <nav className="kit-topbar-nav" aria-label="Main">
        <Link href="/editor" aria-current={current("/editor")}>Editor</Link>
        <Link href="/history" aria-current={current("/history")}>History</Link>
        <a href="https://clients.kritano.com">Brands ↗</a>
      </nav>
    </header>
  );
}
