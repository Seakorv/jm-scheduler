import react from "@vitejs/plugin-react";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
  },
  server: {
    proxy: { "/api": "http://127.0.0.1:3000" },
    fs: { allow: [".", "../shared"] },
  },
  resolve: {
    alias: { "@shared": fileURLToPath(new URL("../shared", import.meta.url)) },
  },
});
