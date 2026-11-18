import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Configuration adaptée à Tauri : port fixe attendu par tauri.conf.json,
// écran non effacé pour garder les journaux Rust visibles.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  build: {
    target: "es2022",
    sourcemap: false,
  },
});
