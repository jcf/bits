import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test('should show login page', async ({ page }) => {
    await page.goto('/login');
    
    await expect(page.getByRole('heading', { name: 'Sign in to your account' })).toBeVisible();
    await expect(page.getByPlaceholder('you@example.com')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Send Magic Link' })).toBeVisible();
  });

  test('should send magic link', async ({ page }) => {
    await page.goto('/login');
    
    // Fill in email
    await page.getByPlaceholder('you@example.com').fill('test@example.com');
    
    // Click send magic link
    await page.getByRole('button', { name: 'Send Magic Link' }).click();
    
    // Should show success message (or console log in dev)
    await expect(page.getByText(/Check your email|Magic link sent/)).toBeVisible({ timeout: 10000 });
  });

  test('should require authentication for upload', async ({ page }) => {
    await page.goto('/upload');
    
    // Should redirect to login or show login prompt
    const url = page.url();
    expect(url).toContain('/login');
  });
});