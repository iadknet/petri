import { expect, test } from "@playwright/test";

test("idle to active controls smoke flow", async ({ page }) => {
  const consoleErrors: string[] = [];
  page.on("console", (msg) => {
    if (msg.type() === "error") {
      consoleErrors.push(msg.text());
    }
  });
  page.on("pageerror", (err) => {
    consoleErrors.push(err.message);
  });

  await page.goto("/");
  await expect(page.getByText("Simulation not started")).toBeVisible();

  const startButton = page.getByRole("button", { name: "Start Simulation", exact: true });
  await expect(startButton).toBeEnabled();
  await startButton.click();

  await expect(page.getByText("Simulation not started")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Pause" })).toBeVisible();

  await page.getByRole("button", { name: "Pause" }).click();
  await expect(page.getByRole("button", { name: "Resume" })).toBeVisible();

  await page.getByRole("button", { name: "Resume" }).click();
  await expect(page.getByRole("button", { name: "Pause" })).toBeVisible();

  await page.getByRole("button", { name: "Restart Simulation", exact: true }).click();
  await expect(page.getByRole("button", { name: "Pause" })).toBeVisible();

  expect(consoleErrors).toEqual([]);
});
