import { expect, test } from "@playwright/test";

test("desktop shell startup/runtime/paint flow", async ({ page }) => {
  let pageOrigin = "";
  const simulationRequestOrigins = new Set<string>();

  const consoleErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") {
      consoleErrors.push(message.text());
    }
  });
  page.on("request", (request) => {
    if (request.url().includes("/v2/simulation/")) {
      simulationRequestOrigins.add(new URL(request.url()).origin);
    }
  });
  page.on("pageerror", (error) => {
    consoleErrors.push(error.message);
  });

  await page.goto("/");
  pageOrigin = new URL(page.url()).origin;

  await expect(page.getByRole("heading", { name: "Petri V2 Control Surface" })).toBeVisible();

  await page.getByRole("button", { name: "Apply Startup" }).click();
  await page.getByRole("button", { name: "Start", exact: true }).click();
  await expect(page.getByText(/State\s+running/)).toBeVisible();
  await page.getByRole("button", { name: "Pause", exact: true }).click();
  await expect(page.getByText(/State\s+paused/)).toBeVisible();

  const stepInput = page.getByLabel("Step Count");
  await stepInput.fill("3");
  const stepButton = page.getByRole("button", { name: "Step", exact: true });
  await expect(stepButton).toBeEnabled();
  await stepButton.click();

  await page.getByRole("button", { name: "Food Tool", exact: true }).click();
  await page.locator("[data-testid='viewport-canvas']").click({ position: { x: 32, y: 32 } });

  await expect(page.getByText("Paint last touched cells:")).toBeVisible();
  expect(simulationRequestOrigins.size).toBeGreaterThan(0);
  expect([...simulationRequestOrigins]).toContain("http://127.0.0.1:4180");
  expect(simulationRequestOrigins.has(pageOrigin)).toBe(false);
  expect(consoleErrors).toEqual([]);
});
