import { test, expect, Page } from "@playwright/test";

// Generate unique email per test run to avoid collisions with existing data
const testEmail = `e2e-${Date.now()}@example.com`;
const testPassword = "securepassword123";

// Helper: fill the registration form.
// The password-toggle.js progressive enhancement removes the confirm password
// field and replaces it with a hidden input that mirrors the password value.
// We wait for the page to settle, then only fill the confirm field if it's
// still visible (i.e., JS hasn't run or hasn't removed it yet).
async function fillRegistrationForm(
  page: Page,
  email: string,
  password: string
) {
  // Wait for the page to fully load (including deferred scripts)
  await page.waitForLoadState("networkidle");

  await page.getByLabel("Email").fill(email);
  await page.getByLabel("Password", { exact: true }).fill(password);

  // After JS runs, the confirm field may be removed.
  // Check if it's still in the DOM and visible.
  const confirmCount = await page
    .getByLabel("Confirm password")
    .count();
  if (confirmCount > 0) {
    const isVisible = await page
      .getByLabel("Confirm password")
      .isVisible();
    if (isVisible) {
      await page.getByLabel("Confirm password").fill(password);
    }
  }

  await page.getByRole("button", { name: "Create account" }).click();
}

test.describe("Core user journey", () => {
  test("visitor sees register and sign in links on index page", async ({
    page,
  }) => {
    await page.goto("/");

    await expect(page).toHaveTitle("Todo List");
    await expect(
      page.getByRole("heading", { name: "Welcome" })
    ).toBeVisible();
    await expect(
      page.getByRole("link", { name: "Create account" }).first()
    ).toBeVisible();
    await expect(
      page.getByRole("link", { name: "Sign in" }).first()
    ).toBeVisible();
  });

  test("full user journey: register, add, complete, delete, logout", async ({
    page,
  }) => {
    // 1. Visit index and click Create account (the visible hero CTA)
    await page.goto("/");
    await page
      .locator(".landing-hero")
      .getByRole("link", { name: "Create account" })
      .click();
    await expect(page).toHaveURL(/\/register/);

    // 2. Fill registration form and submit
    await fillRegistrationForm(page, testEmail, testPassword);

    // Should redirect to login after registration
    await expect(page).toHaveURL(/\/login/);

    // 3. Log in
    await page.getByLabel("Email").fill(testEmail);
    await page.getByLabel("Password").fill(testPassword);
    await page.getByRole("button", { name: "Sign in" }).click();

    // Should redirect to todos
    await expect(page).toHaveURL(/\/todos/);
    await expect(
      page.getByRole("heading", { name: "My Todos" })
    ).toBeVisible();

    // 4. Add a todo
    await page.getByLabel("New todo").fill("Buy groceries");
    await page.getByRole("button", { name: "Add todo" }).click();

    // The todo should appear in the list
    await expect(page.getByText("Buy groceries")).toBeVisible();

    // 5. Complete the todo (click the toggle button)
    await page
      .getByRole("button", { name: /Mark .+Buy groceries.+ as complete/ })
      .click();

    // After toggling, the page should still show the todo
    // (either via HTMX swap or page reload)
    await expect(page.getByText("Buy groceries")).toBeVisible();

    // 6. Delete the todo
    // The delete uses a <details>/<summary> confirmation pattern
    await page
      .locator('[aria-label*="Delete"][aria-label*="Buy groceries"]')
      .click();
    // Click the confirm delete button
    await page.getByRole("button", { name: "Yes" }).click();

    // The todo should be removed
    await expect(page.getByText("Buy groceries")).not.toBeVisible();

    // 7. Logout
    await page.getByRole("button", { name: "Sign out" }).click();

    // Should redirect to login, then we go to index
    await page.goto("/");
    await expect(
      page.getByRole("link", { name: "Create account" }).first()
    ).toBeVisible();
    await expect(
      page.getByRole("link", { name: "Sign in" }).first()
    ).toBeVisible();
  });

  test("welcome page nav links are visually hidden", async ({ page }) => {
    await page.goto("/");

    // The nav links exist in the DOM for screen readers
    const nav = page.locator("nav");
    const navCreateLink = nav.getByRole("link", { name: "Create account" });
    const navSignInLink = nav.getByRole("link", { name: "Sign in" });

    await expect(navCreateLink).toBeAttached();
    await expect(navSignInLink).toBeAttached();

    // But they should be visually hidden (clipped to 1x1 pixel)
    const createBox = await navCreateLink.boundingBox();
    expect(createBox).not.toBeNull();
    expect(createBox!.width).toBeLessThanOrEqual(1);
    expect(createBox!.height).toBeLessThanOrEqual(1);

    const signInBox = await navSignInLink.boundingBox();
    expect(signInBox).not.toBeNull();
    expect(signInBox!.width).toBeLessThanOrEqual(1);
    expect(signInBox!.height).toBeLessThanOrEqual(1);
  });

  test("todo toggle checkbox characters are visible", async ({ page }) => {
    const email = `e2e-toggle-${Date.now()}@example.com`;

    // Register and login
    await page.goto("/register");
    await fillRegistrationForm(page, email, testPassword);
    await page.goto("/login");
    await page.getByLabel("Email").fill(email);
    await page.getByLabel("Password").fill(testPassword);
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page).toHaveURL(/\/todos/);

    // Add a todo so we have a toggle button
    await page.getByLabel("New todo").fill("Test item");
    await page.getByRole("button", { name: "Add todo" }).click();
    await expect(page.getByText("Test item")).toBeVisible();

    // The toggle button should NOT have white text color
    const toggleButton = page.getByRole("button", {
      name: /Mark .+Test item.+ as complete/,
    });
    await expect(toggleButton).toBeVisible();

    const color = await toggleButton.evaluate(
      (el) => getComputedStyle(el).color
    );
    // White is rgb(255, 255, 255) -- toggle should NOT be white
    expect(color).not.toBe("rgb(255, 255, 255)");
  });

  test("authenticated user is redirected from index to todos", async ({
    page,
  }) => {
    const email = `e2e-redirect-${Date.now()}@example.com`;

    // Register
    await page.goto("/register");
    await fillRegistrationForm(page, email, testPassword);

    // Login
    await page.goto("/login");
    await page.getByLabel("Email").fill(email);
    await page.getByLabel("Password").fill(testPassword);
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page).toHaveURL(/\/todos/);

    // Now visit index -- should redirect to /todos
    await page.goto("/");
    await expect(page).toHaveURL(/\/todos/);
  });
});
