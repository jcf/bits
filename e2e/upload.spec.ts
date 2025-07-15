import { test, expect, Page } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

// Helper to create a test file
function createTestFile(filename: string, content: string) {
  const filepath = path.join(__dirname, filename);
  fs.writeFileSync(filepath, content);
  return filepath;
}

// Helper to mock authentication
async function mockAuth(page: Page) {
  // Set a fake auth token in localStorage
  await page.addInitScript(() => {
    localStorage.setItem('authToken', 'test-token');
  });
  
  // Mock the auth check API
  await page.route('**/api/auth/check', async route => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        user: { id: 'test-user', email: 'test@example.com' }
      })
    });
  });
}

test.describe('Content Upload', () => {
  test.beforeEach(async ({ page }) => {
    await mockAuth(page);
  });

  test('should show upload form', async ({ page }) => {
    await page.goto('/upload');
    
    await expect(page.getByRole('heading', { name: 'Upload Content' })).toBeVisible();
    await expect(page.getByLabel('Title')).toBeVisible();
    await expect(page.getByLabel('Description')).toBeVisible();
    await expect(page.getByLabel('Price (USD)')).toBeVisible();
    await expect(page.getByLabel('Content File')).toBeVisible();
  });

  test('should validate required fields', async ({ page }) => {
    await page.goto('/upload');
    
    // Try to submit without filling fields
    await page.getByRole('button', { name: 'Upload Content' }).click();
    
    // HTML5 validation should prevent submission
    const titleInput = page.getByLabel('Title');
    const validationMessage = await titleInput.evaluate(el => (el as HTMLInputElement).validationMessage);
    expect(validationMessage).toBeTruthy();
  });

  test('should handle file upload and encryption', async ({ page }) => {
    await page.goto('/upload');
    
    // Create a test file
    const testContent = 'This is test content for encryption';
    const testFile = createTestFile('test-content.txt', testContent);
    
    // Mock S3 upload URL
    await page.route('**/api/content/upload-url', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          uploadUrl: 'https://test-bucket.s3.amazonaws.com/test-upload-url',
          key: 'content/test-key/test-content.txt'
        })
      });
    });
    
    // Mock S3 upload
    await page.route('https://test-bucket.s3.amazonaws.com/test-upload-url', async route => {
      await route.fulfill({ status: 200 });
    });
    
    // Mock content creation
    await page.route('**/api/content', async route => {
      const body = route.request().postDataJSON();
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'test-content-id',
          title: body.title,
          description: body.description,
          priceCents: body.priceCents
        })
      });
    });
    
    // Fill form
    await page.getByLabel('Title').fill('Test Content');
    await page.getByLabel('Description').fill('This is a test description');
    await page.getByLabel('Price (USD)').fill('9.99');
    
    // Upload file
    await page.getByLabel('Content File').setInputFiles(testFile);
    
    // Submit
    await page.getByRole('button', { name: 'Upload Content' }).click();
    
    // Should redirect to content page
    await expect(page).toHaveURL(/\/content\/test-content-id/, { timeout: 10000 });
    
    // Cleanup
    fs.unlinkSync(testFile);
  });

  test('should show upload progress', async ({ page }) => {
    await page.goto('/upload');
    
    const testFile = createTestFile('large-content.txt', 'x'.repeat(1000000)); // 1MB file
    
    // Mock slow upload
    await page.route('**/api/content/upload-url', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          uploadUrl: 'https://test-bucket.s3.amazonaws.com/test-upload-url',
          key: 'content/test-key/large-content.txt'
        })
      });
    });
    
    await page.route('https://test-bucket.s3.amazonaws.com/test-upload-url', async route => {
      // Simulate slow upload
      await new Promise(resolve => setTimeout(resolve, 2000));
      await route.fulfill({ status: 200 });
    });
    
    // Fill form
    await page.getByLabel('Title').fill('Large Content');
    await page.getByLabel('Description').fill('Large file test');
    await page.getByLabel('Price (USD)').fill('19.99');
    await page.getByLabel('Content File').setInputFiles(testFile);
    
    // Submit
    await page.getByRole('button', { name: 'Upload Content' }).click();
    
    // Should show loading state
    await expect(page.getByText(/Encrypting and uploading/)).toBeVisible();
    
    // Cleanup
    fs.unlinkSync(testFile);
  });
});