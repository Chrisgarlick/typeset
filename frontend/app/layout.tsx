import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Typeset",
  description: "Branded document renderer — Markdown to PDF & DOCX",
  robots: {
    index: false,
    follow: false,
  },
};

// kit (~/dev/kit, `kit dist`) styles the components; its CSS and fonts are in
// public/kit. Light and dark follow the system, as in Clients and Motion.
const KIT_VERSION = "38e203d";

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="h-full">
      <head>
        <meta name="color-scheme" content="light dark" />
        <link rel="stylesheet" href={`/kit/kit.css?v=${KIT_VERSION}`} />
      </head>
      <body className="h-full antialiased">{children}</body>
    </html>
  );
}
