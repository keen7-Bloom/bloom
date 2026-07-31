import { defineConfig } from "vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 0. wallpaper.html is the only page Bloom actually loads — lib.rs builds one
  //    window pointing at it, and all control is via the tray menu. index.html and
  //    src/ are leftover Tauri scaffold (its `greet` command doesn't even exist in
  //    lib.rs); keeping them out of the input means they stop shipping inside every
  //    installer.
  build: {
    rollupOptions: {
      input: {
        wallpaper: "wallpaper.html",
      },
    },
  },
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
