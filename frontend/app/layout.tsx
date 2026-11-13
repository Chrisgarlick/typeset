import type { Metadata } from "next";
// kit (~/dev/kit, `kit dist`), then Typeset's brand as kit tokens and fonts
// (`kit brand pull typeset -into brand`), then Typeset's own layer. The order
// matters: each one overrides the one before.
import "../kit/kit.css";
import "../brand/tokens.css";
import "../brand/fonts.css";
import "./globals.css";
import "./typeset.css";
import mark from "../brand/assets/typeset-mark.svg";

export const metadata: Metadata = {
  title: "Typeset",
  description: "Markdown in, branded PDF out: Markdown to a client's branded PDF or DOCX in about a second.",
  icons: { icon: mark.src },
  robots: {
    index: false,
    follow: false,
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  // Typeset's brand is a light one: grey canvas, white cards, black slabs.
  return (
    <html lang="en" data-theme="light" className="h-full">
      <head>
        <meta name="color-scheme" content="light" />
      </head>
      <body className="min-h-full antialiased">{children}</body>
    </html>
  );
}
