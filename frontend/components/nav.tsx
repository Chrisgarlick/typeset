"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

export function Nav() {
  const path = usePathname();

  const link = (href: string, label: string) => (
    <Link
      href={href}
      className={`text-sm transition-colors ${
        path.startsWith(href)
          ? "text-foreground font-medium"
          : "text-muted hover:text-foreground"
      }`}
    >
      {label}
    </Link>
  );

  return (
    <nav className="flex items-center justify-between border-b border-border px-6 py-3 shrink-0">
      <Link href="/" className="flex items-center gap-2">
        <span className="text-lg font-semibold tracking-tight">Typeset</span>
      </Link>
      <div className="flex items-center gap-5">
        {link("/editor", "Editor")}
        {link("/profiles", "Brands")}
        {link("/history", "History")}
      </div>
    </nav>
  );
}
