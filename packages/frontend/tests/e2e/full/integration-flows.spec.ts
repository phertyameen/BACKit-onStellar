import { test, expect } from '@playwright/test';
import { TestHelpers, ApiMocks, TEST_WALLET_ADDRESS, TEST_CALL_DATA } from './utils';

test.describe('Critical User Flows Integration Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Set up all necessary API mocks
    await ApiMocks.mockTokensApi(page);
    await ApiMocks.mockFeedApi(page);
    await ApiMocks.mockUserProfileApi(page);
    
    // Mock call creation API
    await page.route('/api/calls/create', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'integration-test-call-123',
          success: true,
          xdr: 'AAAA...'
        })
      });
    });
    
    // Mock staking API
    await page.route('/api/calls/integration-test-call-123/stake', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          xdr: 'AAAA...',
          success: true
        })
      });
    });
    
    // Connect wallet
    await TestHelpers.connectWallet(page);
  });

  test('Complete flow: Connect wallet → Create call → Verify in feed', async ({ page }) => {
    // Step 1: Wallet should already be connected from beforeEach
    
    // Step 2: Navigate to create call page
    await page.goto('/create');
    
    // Step 3: Create a new call
    await page.getByPlaceholder('Search tokens...').fill(TEST_CALL_DATA.token);
    await page.getByText(TEST_CALL_DATA.token).first().click();
    await page.getByPlaceholder('Enter condition').fill(TEST_CALL_DATA.condition);
    await page.getByPlaceholder('Stake amount (USDC)').fill(TEST_CALL_DATA.stake.toString());
    await page.getByPlaceholder('End time').fill(TEST_CALL_DATA.endTime);
    await page.getByPlaceholder('Thesis (Markdown supported)').fill(TEST_CALL_DATA.thesis);
    await page.getByRole('button', { name: 'Create Call' }).click();
    
    // Step 4: Wait for navigation to call detail page
    await page.waitForURL(/\/call\/.+/);
    const callUrl = page.url();
    const callId = callUrl.split('/').pop();
    
    // Step 5: Navigate to feed
    await page.goto('/feed');
    
    // Step 6: Verify the newly created call appears in feed
    // Note: This depends on your feed implementation
    // You might need to refresh or wait for the call to appear
    await page.waitForTimeout(2000);
    
    // Look for the call in the feed
    await expect(page.getByText(TEST_CALL_DATA.condition)).toBeVisible();
  });

  test('Complete flow: View call → Stake YES → Verify stake shown', async ({ page }) => {
    // Step 1: Navigate to a call (using mocked feed data)
    await page.goto('/feed');
    
    // Step 2: Click on a call to view details
    // This assumes the first call in feed is clickable
    await page.getByText('BTC > $50K by Jan 2026').first().click();
    
    // Step 3: Verify we're on call detail page
    await page.waitForURL(/\/call\/.+/);
    
    // Step 4: Stake YES on the call
    await page.getByRole('button', { name: 'YES' }).click();
    await page.getByPlaceholder('Amount (USDC)').fill('75');
    await page.getByRole('button', { name: 'Stake' }).click();
    
    // Step 5: Wait for transaction processing
    await page.waitForTimeout(2000);
    
    // Step 6: Verify stake was recorded and displayed
    // This depends on your UI implementation
    // You might show the user's stake in a "Your Stakes" section
    // or update the stake visualization
    
    // Check if stake visualization updated
    const yesStakeElement = page.locator('[data-testid="yes-stake"]');
    if (await yesStakeElement.count() > 0) {
      const stakeAmount = await yesStakeElement.textContent();
      expect(stakeAmount).toContain('75'); // Should include our 75 USDC stake
    }
    
    // Alternative: Check for success message
    await expect(page.getByText('Staked 75 USDC on YES')).toBeVisible();
  });

  test('Complete flow: View profile → Check stats', async ({ page }) => {
    // Step 1: Navigate to profile page
    await page.goto(`/profile/${TEST_WALLET_ADDRESS}`);
    
    // Step 2: Verify profile header information
    await expect(page.getByText('Test User')).toBeVisible();
    await expect(page.getByText(TEST_WALLET_ADDRESS.substring(0, 10))).toBeVisible();
    
    // Step 3: Check profile statistics
    await expect(page.getByText('Win Rate')).toBeVisible();
    await expect(page.getByText('75%')).toBeVisible();
    
    await expect(page.getByText('Total Calls')).toBeVisible();
    await expect(page.getByText('12')).toBeVisible();
    
    await expect(page.getByText('Followers')).toBeVisible();
    await expect(page.getByText('42')).toBeVisible();
    
    await expect(page.getByText('Following')).toBeVisible();
    await expect(page.getByText('18')).toBeVisible();
    
    // Step 4: Navigate through different tabs
    await page.getByRole('tab', { name: 'Created' }).click();
    await page.getByRole('tab', { name: 'Participated' }).click();
    await page.getByRole('tab', { name: 'Resolved' }).click();
    await page.getByRole('tab', { name: 'Created' }).click();
    
    // Step 5: Verify tab content loads
    await expect(page.getByRole('tab', { name: 'Created' })).toHaveAttribute('aria-selected', 'true');
  });

  test('End-to-end flow: Create call → Stake on own call → View updated profile', async ({ page }) => {
    // Step 1: Create a new call
    await page.goto('/create');
    await page.getByPlaceholder('Search tokens...').fill('ETH');
    await page.getByText('ETH').first().click();
    await page.getByPlaceholder('Enter condition').fill('ETH > $3000 by Feb 2026');
    await page.getByPlaceholder('Stake amount (USDC)').fill('200');
    await page.getByPlaceholder('End time').fill('2026-02-15T12:00');
    await page.getByPlaceholder('Thesis (Markdown supported)').fill('Ethereum will surge due to ETF approval');
    await page.getByRole('button', { name: 'Create Call' }).click();
    
    // Step 2: Wait for navigation and get call ID
    await page.waitForURL(/\/call\/.+/);
    const callUrl = page.url();
    const callId = callUrl.split('/').pop();
    
    // Step 3: Stake on our own call
    await page.getByRole('button', { name: 'YES' }).click();
    await page.getByPlaceholder('Amount (USDC)').fill('100');
    await page.getByRole('button', { name: 'Stake' }).click();
    await page.waitForTimeout(2000);
    
    // Step 4: Navigate to profile
    await page.goto(`/profile/${TEST_WALLET_ADDRESS}`);
    
    // Step 5: Check that total calls count increased
    // Note: This would require updating the mock to reflect the new call
    // In a real implementation, you'd see the call in the "Created" tab
    await page.getByRole('tab', { name: 'Created' }).click();
    
    // Step 6: Verify the new call appears in created calls
    await expect(page.getByText('ETH > $3000 by Feb 2026')).toBeVisible();
  });

  test('Navigation flow: Home → Feed → Profile → Create → Call Detail', async ({ page }) => {
    // Step 1: Start at home page
    await page.goto('/');
    await expect(page.getByText('BACKit on Stellar')).toBeVisible();
    
    // Step 2: Navigate to feed
    await page.goto('/feed');
    await expect(page.getByText('For You')).toBeVisible();
    
    // Step 3: Navigate to profile
    await page.goto(`/profile/${TEST_WALLET_ADDRESS}`);
    await expect(page.getByText('Test User')).toBeVisible();
    
    // Step 4: Navigate to create call
    await page.goto('/create');
    await expect(page.getByRole('button', { name: 'Create Call' })).toBeVisible();
    
    // Step 5: Navigate back to a call detail (from feed)
    await page.goto('/feed');
    await page.getByText('BTC > $50K by Jan 2026').first().click();
    await page.waitForURL(/\/call\/.+/);
    await expect(page.getByText('BTC > $50000 by Jan 2026')).toBeVisible();
    
    // Step 6: Verify navigation history works
    await page.goBack(); // Back to feed
    await expect(page.getByText('For You')).toBeVisible();
    
    await page.goBack(); // Back to create
    await expect(page.getByRole('button', { name: 'Create Call' })).toBeVisible();
  });
});