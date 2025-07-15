import { test, expect } from '@playwright/test';

test.describe('Content Purchase', () => {
  test('should show content details', async ({ page }) => {
    // Mock content detail
    await page.route('**/api/content/test-id', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'test-id',
          title: 'Premium Content',
          description: 'This is premium content that requires payment',
          priceCents: 1999,
          encryptedUrl: 'https://example.com/encrypted.dat',
          previewUrl: 'https://example.com/preview.jpg'
        })
      });
    });
    
    await page.goto('/content/test-id');
    
    await expect(page.getByRole('heading', { name: 'Premium Content' })).toBeVisible();
    await expect(page.getByText('This is premium content that requires payment')).toBeVisible();
    await expect(page.getByText('$19.99')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Purchase Access' })).toBeVisible();
  });

  test('should require login for purchase', async ({ page }) => {
    // Mock content detail
    await page.route('**/api/content/test-id', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'test-id',
          title: 'Premium Content',
          description: 'Login required',
          priceCents: 999,
          encryptedUrl: 'https://example.com/encrypted.dat'
        })
      });
    });
    
    await page.goto('/content/test-id');
    
    // Click purchase without auth
    await page.getByRole('button', { name: 'Purchase Access' }).click();
    
    // Should redirect to login
    await expect(page).toHaveURL('/login');
  });

  test('should handle purchase flow', async ({ page }) => {
    // Mock authentication
    await page.addInitScript(() => {
      localStorage.setItem('authToken', 'test-token');
    });
    
    // Mock content detail
    await page.route('**/api/content/test-id', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'test-id',
          title: 'Premium Content',
          description: 'Purchase me',
          priceCents: 999,
          encryptedUrl: 'https://example.com/encrypted.dat'
        })
      });
    });
    
    // Mock purchase check (not purchased)
    let purchaseCount = 0;
    await page.route('**/api/content/test-id/key', async route => {
      if (purchaseCount === 0) {
        await route.fulfill({ status: 403 });
      } else {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            key: 'test-encryption-key',
            iv: 'test-iv'
          })
        });
      }
    });
    
    // Mock payment intent
    await page.route('**/api/purchase/intent', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          clientSecret: 'test-client-secret',
          amount: 999
        })
      });
    });
    
    // Mock purchase confirmation
    await page.route('**/api/purchase/confirm', async route => {
      purchaseCount++;
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ success: true })
      });
    });
    
    await page.goto('/content/test-id');
    
    // Click purchase
    await page.getByRole('button', { name: 'Purchase Access' }).click();
    
    // Should show processing
    await expect(page.getByText('Processing...')).toBeVisible();
    
    // Should show success message
    await expect(page.getByText('You have purchased this content. Enjoy!')).toBeVisible({ timeout: 10000 });
  });

  test('should decrypt content after purchase', async ({ page }) => {
    // Mock authentication
    await page.addInitScript(() => {
      localStorage.setItem('authToken', 'test-token');
    });
    
    // Mock content detail
    await page.route('**/api/content/test-id', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'test-id',
          title: 'Encrypted Image',
          description: 'This image is encrypted',
          priceCents: 999,
          encryptedUrl: 'https://example.com/encrypted.jpg'
        })
      });
    });
    
    // Mock already purchased
    await page.route('**/api/content/test-id/key', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          key: btoa('test-encryption-key-32-bytes-long'),
          iv: btoa('test-iv-12by')
        })
      });
    });
    
    // Mock encrypted content
    await page.route('https://example.com/encrypted.jpg', async route => {
      // Return some dummy encrypted data
      const buffer = Buffer.from('encrypted-content-data');
      await route.fulfill({
        status: 200,
        contentType: 'application/octet-stream',
        body: buffer
      });
    });
    
    await page.goto('/content/test-id');
    
    // Should show purchased state
    await expect(page.getByText('You have purchased this content. Enjoy!')).toBeVisible();
    
    // In a real test, we'd verify the decryption happened
    // For now, we just check that no error occurred
    await expect(page.getByText('Failed to decrypt content')).not.toBeVisible();
  });
});