import net from "node:net";

async function claimPort(preferredPort?: number): Promise<number> {
	return new Promise((resolve, reject) => {
		const server = net.createServer();
		server.unref();

		server.once("error", (error) => {
			reject(error);
		});

		server.listen(preferredPort ?? 0, "127.0.0.1", () => {
			const address = server.address();
			if (!address || typeof address === "string") {
				server.close();
				reject(new Error("Unable to resolve allocated port"));
				return;
			}
			const port = address.port;
			server.close((closeError) => {
				if (closeError) {
					reject(closeError);
					return;
				}
				resolve(port);
			});
		});
	});
}

export interface PortAllocationOptions {
	fixedBackendPort?: number;
	fixedFrontendPort?: number;
}

type ClaimPortFn = (preferredPort?: number) => Promise<number>;

export async function allocatePorts(
	options: PortAllocationOptions = {},
	claim: ClaimPortFn = claimPort,
): Promise<{ backendPort: number; frontendPort: number }> {
	const { fixedBackendPort, fixedFrontendPort } = options;

	if (
		fixedBackendPort !== undefined &&
		fixedFrontendPort !== undefined &&
		fixedBackendPort === fixedFrontendPort
	) {
		throw new Error("E2E backend and frontend ports must differ");
	}

	const backendPort = await claim(fixedBackendPort);
	let frontendPort = await claim(fixedFrontendPort);

	while (frontendPort === backendPort) {
		frontendPort = await claim();
	}

	return { backendPort, frontendPort };
}
