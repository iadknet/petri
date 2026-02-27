import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const proxyTarget = process.env.VITE_PROXY_TARGET ?? "http://localhost:3000";
const apiProxy = {
	"/v3": {
		target: proxyTarget,
		changeOrigin: true,
		ws: true,
	},
};

export default defineConfig({
	plugins: [react(), tailwindcss()],
	server: {
		port: 5173,
		proxy: apiProxy,
	},
	preview: {
		port: 5173,
		proxy: apiProxy,
	},
});
