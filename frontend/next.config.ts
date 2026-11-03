import type { NextConfig } from "next";

// The browser reaches the API through the server-side proxy in
// app/ui-api/[...path]/route.ts, which adds the token.
const nextConfig: NextConfig = {
  output: "standalone",
};

export default nextConfig;
