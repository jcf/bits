import { test, expect } from '@playwright/test';

test.describe('Browse Content', () => {
  test('should show browse page', async ({ page }) => {
    await page.goto('/');
    
    await expect(page.getByRole('heading', { name: 'Browse Content' })).toBeVisible();
  });

  test('should show empty state when no content', async ({ page }) => {
    // Mock empty content response
    await page.route('**/api/content', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify([])
      });
    });
    
    await page.goto('/');
    
    await expect(page.getByText('No content available yet.')).toBeVisible();
    await expect(page.getByText('Be the first to upload content!')).toBeVisible();
  });

  test('should display content grid', async ({ page }) => {
    // Mock content response
    await page.route('**/api/content', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify([
          {
            id: '1',
            title: 'Test Content 1',
            description: 'Description 1',
            priceCents: 999,
            createdAt: new Date().toISOString()
          },
          {
            id: '2',
            title: 'Test Content 2',
            description: 'Description 2',
            priceCents: 1999,
            previewUrl: 'https://example.com/preview.jpg',
            createdAt: new Date().toISOString()
          }
        ])
      });
    });
    
    await page.goto('/');
    
    // Check content cards
    await expect(page.getByText('Test Content 1')).toBeVisible();
    await expect(page.getByText('Test Content 2')).toBeVisible();
    await expect(page.getByText('$9.99')).toBeVisible();
    await expect(page.getByText('$19.99')).toBeVisible();
  });

  test('should navigate to content detail page', async ({ page }) => {
    // Mock content response
    await page.route('**/api/content', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify([
          {
            id: 'test-id',
            title: 'Clickable Content',
            description: 'Click me',
            priceCents: 999,
            createdAt: new Date().toISOString()
          }
        ])
      });
    });
    
    await page.goto('/');
    
    // Click on content
    await page.getByText('Clickable Content').click();
    
    // Should navigate to detail page
    await expect(page).toHaveURL('/content/test-id');
  });
});