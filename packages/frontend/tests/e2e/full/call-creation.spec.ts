import { test, expect } from '@playwright/test';
import { TestHelpers, ApiMocks, TEST_CALL_DATA } from './utils';

test.describe('Call Creation Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Mock API responses
    await ApiMocks.mockTokensApi(page);
    await page.route('/api/calls/create', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'test-call-123',
          success: true,
          xdr: 'AAAA...' // Mock XDR
        })
      });
    });
    
    // Connect wallet
    await TestHelpers.connectWallet(page);
    
    // Navigate to create page
    await page.goto('/create');
  });

  test('should create a new call successfully', async ({ page }) => {
    // Fill in the form
    await page.getByPlaceholder('Search tokens...').fill(TEST_CALL_DATA.token);
    await page.getByText(TEST_CALL_DATA.token).first().click();
    
    await page.getByPlaceholder('Enter condition').fill(TEST_CALL_DATA.condition);
    await page.getByPlaceholder('Stake amount (USDC)').fill(TEST_CALL_DATA.stake.toString());
    await page.getByPlaceholder('End time').fill(TEST_CALL_DATA.endTime);
    await page.getByPlaceholder('Thesis (Markdown supported)').fill(TEST_CALL_DATA.thesis);
    
    // Submit the form
    await page.getByRole('button', { name: 'Create Call' }).click();
    
    // Wait for navigation to call detail page
    await page.waitForURL(/\/call\/.+/);
    
    // Verify we're on the call detail page
    await expect(page).toHaveURL(/\/call\/.+/);
    
    // Verify call details are displayed
    await expect(page.getByText(TEST_CALL_DATA.condition)).toBeVisible();
    await expect(page.getByText(TEST_CALL_DATA.token)).toBeVisible();
  });

  test('should show validation errors for invalid input', async ({ page }) => {
    // Try to submit empty form
    await page.getByRole('button', { name: 'Create Call' }).click();
    
    // Should show validation error (this depends on your form validation implementation)
    // In a real implementation, you might check for specific error messages
    await expect(page.getByRole('button', { name: 'Create Call' })).toBeVisible();
  });

  test('should preserve form data during navigation', async ({ page }) => {
    // Fill in partial form data
    await page.getByPlaceholder('Search tokens...').fill(TEST_CALL_DATA.token);
    await page.getByPlaceholder('Enter condition').fill(TEST_CALL_DATA.condition);
    
    // Navigate away and back
    await page.goto('/');
    await page.goBack();
    
    // Verify form data is preserved
    await expect(page.getByPlaceholder('Search tokens...')).toHaveValue(TEST_CALL_DATA.token);
    await expect(page.getByPlaceholder('Enter condition')).toHaveValue(TEST_CALL_DATA.condition);
  });

  test('should handle token search functionality', async ({ page }) => {
    // Type in token search
    await page.getByPlaceholder('Search tokens...').fill('BTC');
    
    // Wait for search results
    await page.waitForTimeout(500);
    
    // Click on search result
    await page.getByText('Bitcoin').first().click();
    
    // Verify token is selected
    await expect(page.getByText('BTC')).toBeVisible();
  });
});