import { expect, test } from "@playwright/test";

test("customer assessment displays Northstar and sanitized Geek mode traces", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle("Claim of Thrones | Insurance support");
  await expect(page.getByText("Northstar", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Assess request" })).toBeDisabled();
  await page.getByRole("button", { name: "Geek mode: Off" }).click();
  await expect(page.locator(".conversation")).toHaveClass(/geek-mode/);
  await page.getByRole("checkbox", { name: /fictional information/ }).check();
  await page.getByRole("button", { name: "Assess request" }).click();
  await expect(page.getByRole("heading", { name: "Assessment results" })).toBeVisible();
  await expect(page.getByText("This message may relate to the insurance areas listed below.")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Execution trace" })).toBeVisible();
  await expect(page.locator(".provider-result .execution-trace li")).toHaveCount(5);
  await expect(page.locator("body")).not.toContainText("raw provider exchange");
  await page.getByRole("button", { name: "Employee", exact: true }).click();
  await expect(page.getByRole("heading", { name: /run-.*review required/ }).first()).toBeVisible();
  await page.setViewportSize({ width: 390, height: 844 });
  await page.getByRole("button", { name: "Customer", exact: true }).click();
  await expect(page.getByRole("button", { name: "Assess request" })).toBeVisible();
  await page.screenshot({ path: "artifacts/customer-mobile.png", fullPage: true });
});

test("unknown input reaches Rust and requires clarification instead of fabricated tasks", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "General question", exact: true }).click();
  await page.getByRole("checkbox", { name: /fictional information/ }).check();
  await page.getByRole("button", { name: "Assess request" }).click();
  await expect(page.getByText(/We couldn't identify a specific insurance need/)).toBeVisible();
  await expect(page.locator(".provider-result .tasks li")).toHaveCount(0);
});
