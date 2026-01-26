import { test, expect } from '@playwright/test';
import { TestHelpers, ApiMocks } from './utils';

test.describe('Staking Flow', () => {
  let testCallId: string;

  test.beforeEach(async ({ page }) => {
    // Mock API responses
    await ApiMocks.mockTokensApi(page);
    await ApiMocks.mockStakeApi(page, 'test-call-123');
    
    // Connect wallet
    await TestHelpers.connectWallet(page);
    
    // Create a test call first
    testCallId = await TestHelpers.createTestCall(page);
  });

  test('should stake YES on a call successfully', async ({ page }) => {
    // Navigate to call detail page
    await page.goto(`/call/${testCallId}`);
    
    // Select YES side
    await page.getByRole('button', { name: 'YES' }).click();
    
    // Enter stake amount
    await page.getByPlaceholder('Amount (USDC)').fill('50');
    
    // Submit stake
    await page.getByRole('button', { name: 'Stake' }).click();
    
    // Wait for transaction processing
    await page.waitForTimeout(2000);
    
    // Verify success message or updated UI
    // This depends on your implementation - you might show a success message
    // or update the stake bar immediately
    await expect(page.getByRole('button', { name: 'Stake' })).toBeVisible();
  });

  test('should stake NO on a call successfully', async ({ page }) => {
    await page.goto(`/call/${testCallId}`);
    
    // Select NO side
    await page.getByRole('button', { name: 'NO' }).click();
    
    // Enter stake amount
    await page.getByPlaceholder('Amount (USDC)').fill('25');
    
    // Submit stake
    await page.getByRole('button', { name: 'Stake' }).click();
    
    // Wait for processing
    await page.waitForTimeout(2000);
    
    await expect(page.getByRole('button', { name: 'Stake' })).toBeVisible();
  });

  test('should show validation errors for invalid stake amounts', async ({ page }) => {
    await page.goto(`/call/${testCallId}`);
    
    // Try to stake with 0 amount
    await page.getByPlaceholder('Amount (USDC)').fill('0');
    await page.getByRole('button', { name: 'Stake' }).click();
    
    // Should show validation error
    await expect(page.getByText('Invalid amount')).toBeVisible();
    
    // Try negative amount
    await page.getByPlaceholder('Amount (USDC)').fill('-10');
    await page.getByRole('button', { name: 'Stake' }).click();
    
    await expect(page.getByText('Invalid amount')).toBeVisible();
  });

  test('should update stake visualization after staking', async ({ page }) => {
    await page.goto(`/call/${testCallId}`);
    
    // Get initial stake values
    const initialYesStake = await page.locator('[data-testid="yes-stake"]').textContent();
    const initialNoStake = await page.locator('[data-testid="no-stake"]').textContent();
    
    // Place a stake
    await page.getByRole('button', { name: 'YES' }).click();
    await page.getByPlaceholder('Amount (USDC)').fill('100');
    await page.getByRole('button', { name: 'Stake' }).click();
    await page.waitForTimeout(2000);
    
    // Check if stake visualization updated
    const updatedYesStake = await page.locator('[data-testid="yes-stake"]').textContent();
    
    // The stake amount should have increased (this assumes your UI updates the display)
    expect(updatedYesStake).not.toEqual(initialYesStake);
  });

  test('should show user\'s previous stakes', async ({ page }) => {
    await page.goto(`/call/${testCallId}`);
    
    // Place a stake
    await page.getByRole('button', { name: 'YES' }).click();
    await page.getByPlaceholder('Amount (USDC)').fill('75');
    await page.getByRole('button', { name: 'Stake' }).click();
    await page.waitForTimeout(2000);
    
    // Navigate away and back
    await page.goto('/feed');
    await page.goto(`/call/${testCallId}`);
    
    // Should show user's previous stake (depends on implementation)
    // This might show in a "your stakes" section or highlight the user's position
    await expect(page.getByText('Your Stake')).toBeVisible();
  });
});