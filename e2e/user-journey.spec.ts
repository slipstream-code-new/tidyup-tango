import { test, expect, Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

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

    // Should redirect to dashboard
    await expect(page).toHaveURL(/\/dashboard/);
    await expect(
      page.getByRole("heading", { name: "Dashboard" })
    ).toBeVisible();

    // Navigate to todos page to manage todo items
    await page.goto("/todos");

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

  test("welcome page nav links are accessible but not visible", async ({
    page,
  }) => {
    await page.goto("/");
    const nav = page.locator("nav");
    const navCreateLink = nav.getByRole("link", { name: "Create account" });
    const navSignInLink = nav.getByRole("link", { name: "Sign in" });

    // Accessible to screen readers (in accessibility tree)
    await expect(navCreateLink).toBeAttached();
    await expect(navSignInLink).toBeAttached();

    // Visually hidden via the .visually-hidden class (clip-path technique)
    await expect(navCreateLink).toHaveClass(/visually-hidden/);
    await expect(navSignInLink).toHaveClass(/visually-hidden/);
  });

  test("index page has no automatically detectable a11y violations", async ({
    page,
  }) => {
    await page.goto("/");

    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
      .analyze();

    expect(
      results.violations,
      JSON.stringify(results.violations, null, 2)
    ).toEqual([]);
  });

  test("todos page has no automatically detectable a11y violations", async ({
    page,
  }) => {
    const email = `e2e-axe-${Date.now()}@example.com`;

    // Register and login
    await page.goto("/register");
    await fillRegistrationForm(page, email, testPassword);
    await page.goto("/login");
    await page.getByLabel("Email").fill(email);
    await page.getByLabel("Password").fill(testPassword);
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page).toHaveURL(/\/dashboard/);

    // Navigate to todos page
    await page.goto("/todos");

    // Add a todo so we test the page with content
    await page.getByLabel("New todo").fill("Test item");
    await page.getByRole("button", { name: "Add todo" }).click();
    await expect(page.getByText("Test item")).toBeVisible();

    const pendingResults = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
      .analyze();

    expect(
      pendingResults.violations,
      JSON.stringify(pendingResults.violations, null, 2)
    ).toEqual([]);

    // Toggle the todo to completed and scan again -- completed items
    // use strikethrough + muted color which may have different contrast
    await page
      .getByRole("button", { name: /Mark .+Test item.+ as complete/ })
      .click();
    await expect(page.getByText("Test item")).toBeVisible();

    const completedResults = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
      .analyze();

    expect(
      completedResults.violations,
      JSON.stringify(completedResults.violations, null, 2)
    ).toEqual([]);
  });

  test("dashboard page has no automatically detectable a11y violations", async ({
    page,
  }) => {
    const email = `e2e-axe-dash-${Date.now()}@example.com`;

    // Register and login
    await page.goto("/register");
    await fillRegistrationForm(page, email, testPassword);
    await page.goto("/login");
    await page.getByLabel("Email").fill(email);
    await page.getByLabel("Password").fill(testPassword);
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page).toHaveURL(/\/dashboard/);

    // Dashboard should have proper heading and nav
    await expect(
      page.getByRole("heading", { name: "Dashboard" })
    ).toBeVisible();
    await expect(
      page.getByRole("navigation", { name: "GTD lists" })
    ).toBeVisible();

    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
      .analyze();

    expect(
      results.violations,
      JSON.stringify(results.violations, null, 2)
    ).toEqual([]);
  });

  test("inbox capture: add item, see it listed, delete it", async ({
    page,
  }) => {
    const email = `e2e-inbox-${Date.now()}@example.com`;

    // Register and login
    await page.goto("/register");
    await fillRegistrationForm(page, email, testPassword);
    await page.goto("/login");
    await page.getByLabel("Email").fill(email);
    await page.getByLabel("Password").fill(testPassword);
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page).toHaveURL(/\/dashboard/);

    // Navigate to inbox via direct URL (bypass hx-boost)
    await page.goto("/inbox");
    await page.waitForLoadState("networkidle");
    await expect(
      page.getByRole("heading", { name: "Inbox" })
    ).toBeVisible();

    // Empty state should show
    await expect(page.getByText("Inbox zero")).toBeVisible();

    // Capture an item using the inbox page's main capture form
    const captureInput = page.locator("#inbox-title");
    await expect(captureInput).toBeVisible();
    await captureInput.fill("Call the dentist");
    await page.locator("button.inbox-capture__submit").click();

    // Item should appear (via HTMX swap or page reload)
    await expect(page.getByText("Call the dentist", { exact: true })).toBeVisible();

    // Delete the item
    await page.getByRole("button", { name: "Trash: Call the dentist" }).click();

    // Item should be gone
    await expect(page.getByText("Call the dentist", { exact: true })).not.toBeVisible();
  });

  test("inbox page has no automatically detectable a11y violations", async ({
    page,
  }) => {
    const email = `e2e-axe-inbox-${Date.now()}@example.com`;

    // Register and login
    await page.goto("/register");
    await fillRegistrationForm(page, email, testPassword);
    await page.goto("/login");
    await page.getByLabel("Email").fill(email);
    await page.getByLabel("Password").fill(testPassword);
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page).toHaveURL(/\/dashboard/);

    // Navigate to inbox
    await page.goto("/inbox");

    // Scan empty inbox
    const emptyResults = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
      .analyze();

    expect(
      emptyResults.violations,
      JSON.stringify(emptyResults.violations, null, 2)
    ).toEqual([]);

    // Add an item and scan again (populated state)
    await page.goto("/inbox");
    await page.waitForLoadState("networkidle");
    const inboxInput = page.locator("#inbox-title");
    await expect(inboxInput).toBeVisible();
    await inboxInput.fill("Test a11y item");
    await page.locator("button.inbox-capture__submit").click();
    await expect(page.getByText("Test a11y item", { exact: true })).toBeVisible();

    const withItemResults = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
      .analyze();

    expect(
      withItemResults.violations,
      JSON.stringify(withItemResults.violations, null, 2)
    ).toEqual([]);
  });

  test("context management: view defaults, add, edit, delete", async ({
    page,
  }) => {
    const email = `e2e-ctx-${Date.now()}@example.com`;

    // Register and login
    await page.goto("/register");
    await fillRegistrationForm(page, email, testPassword);
    await page.goto("/login");
    await page.getByLabel("Email").fill(email);
    await page.getByLabel("Password").fill(testPassword);
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page).toHaveURL(/\/dashboard/);

    // Navigate to contexts page
    await page.goto("/contexts");
    await expect(
      page.getByRole("heading", { name: "Contexts" })
    ).toBeVisible();

    // Default contexts should be present (use exact match to avoid matching description text)
    await expect(page.getByText("@computer", { exact: true })).toBeVisible();
    await expect(page.getByText("@home", { exact: true })).toBeVisible();
    await expect(page.getByText("@errands", { exact: true })).toBeVisible();
    await expect(page.getByText("@phone", { exact: true })).toBeVisible();
    await expect(page.getByText("@anywhere", { exact: true })).toBeVisible();

    // Add a new context
    const addInput = page.locator("#context-name");
    await expect(addInput).toBeVisible();
    await addInput.fill("@office");
    await page.getByRole("button", { name: "Add" }).click();

    // New context should appear
    await expect(page.getByText("@office")).toBeVisible();

    // Delete the new context
    await page.getByRole("button", { name: "Delete: @office" }).click();
    await expect(page.getByText("@office")).not.toBeVisible();
  });

  test("contexts page has no automatically detectable a11y violations", async ({
    page,
  }) => {
    const email = `e2e-axe-ctx-${Date.now()}@example.com`;

    // Register and login
    await page.goto("/register");
    await fillRegistrationForm(page, email, testPassword);
    await page.goto("/login");
    await page.getByLabel("Email").fill(email);
    await page.getByLabel("Password").fill(testPassword);
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page).toHaveURL(/\/dashboard/);

    // Navigate to contexts page
    await page.goto("/contexts");

    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
      .analyze();

    expect(
      results.violations,
      JSON.stringify(results.violations, null, 2)
    ).toEqual([]);
  });

  test("authenticated user is redirected from index to dashboard", async ({
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
    await expect(page).toHaveURL(/\/dashboard/);

    // Now visit index -- should redirect to /dashboard
    await page.goto("/");
    await expect(page).toHaveURL(/\/dashboard/);
  });
});
