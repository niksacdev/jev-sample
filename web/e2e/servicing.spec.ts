import { expect, test } from "@playwright/test";

test("real customer message reaches Rust and operator sees the same run", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("button", { name: "Send to Rue" })).toBeDisabled();
  await page.getByRole("checkbox").check();
  await page.getByRole("button", { name: "Send to Rue" }).click();
  await expect(page.getByRole("status")).toContainText("I've organized your request");
  await expect(page.getByRole("status")).not.toContainText("experimental slice");
  await expect(page.getByText("You're trying a preview", { exact: false })).toBeVisible();
  await expect(page.getByRole("status").getByText("Review required", { exact: true })).toHaveCount(4);
  await expect(page.locator("body")).not.toContainText("jev-");
  await page.getByRole("button", { name: "Employee", exact: true }).click();
  await expect(page.getByRole("heading", { name: /run-.*review required/ }).first()).toBeVisible();
  await page.getByRole("button", { name: "Operator", exact: true }).click();
  await expect(page.getByText(/keyword_baseline \/ No model/).first()).toBeVisible();
  await expect(page.getByText("Unmeasured", { exact: true })).toBeVisible();
  await page.screenshot({ path: "artifacts/operator-desktop.png", fullPage: true });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.getByRole("button", { name: "Customer", exact: true }).click();
  await expect(page.getByRole("button", { name: "Send to Rue" })).toBeVisible();
  await page.screenshot({ path: "artifacts/customer-mobile.png", fullPage: true });
});

test("unknown input reaches Rust and requires clarification instead of fabricated tasks", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Clarification", exact: true }).click();
  await page.getByRole("checkbox").check();
  await page.getByRole("button", { name: "Send to Rue" }).click();
  await expect(page.getByRole("status")).toContainText("Please clarify");
  await expect(page.getByRole("status").locator(".tasks li")).toHaveCount(0);
});
