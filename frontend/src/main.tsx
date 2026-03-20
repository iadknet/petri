import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App.tsx";
import "./styles.css";

const queryClient = new QueryClient({
	defaultOptions: {
		queries: {
			retry: 1,
			staleTime: 5_000,
		},
	},
});

// Handle stale hashed chunks after a new frontend build/deploy.
// Without this, lazy imports can fail with a 404 until the user manually refreshes.
window.addEventListener("vite:preloadError", (event) => {
	event.preventDefault();
	window.location.reload();
});

if (import.meta.env.DEV) {
	void import("./testing/e2eHooks.ts").then(({ installE2ETestHooks }) => {
		installE2ETestHooks();
	});
}

createRoot(document.getElementById("root")!).render(
	<StrictMode>
		<QueryClientProvider client={queryClient}>
			<App />
		</QueryClientProvider>
	</StrictMode>,
);
