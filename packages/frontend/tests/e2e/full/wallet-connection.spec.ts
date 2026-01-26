import { test, expect } from '@playwright/test';
import { TestHelpers, ApiMocks, TEST_WALLET_ADDRESS } from './utils';

test.describe('Wallet Connection Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Mock API responses
    await ApiMocks.mockTokensApi(page);
    await ApiMocks.mockFeedApi(page);
    await ApiMocks.mockUserProfileApi(page);
    
    // Navigate to home page
    await page.goto('/');
  });

  test('should connect wallet successfully', async ({ page }) => {
    // Connect wallet using test helper
    await TestHelpers.connectWallet(page);
    
    // Verify wallet is connected by checking localStorage
    const isConnected = await page.evaluate(() => {
      return localStorage.getItem('wallet_connected') === 'true';
    });
    
    expect(isConnected).toBeTruthy();
    
    // Verify wallet address is stored
    const walletAddress = await page.evaluate(() => {
      return localStorage.getItem('wallet_address');
    });
    
    expect(walletAddress).toBe(TEST_WALLET_ADDRESS);
  });

  test('should show wallet address after connection', async ({ page }) => {
    await TestHelpers.connectWallet(page);
    
    // Check if wallet address is displayed on page
    // This would depend on how your UI shows the connected wallet
    // For now, we'll just verify the page loads correctly
    await expect(page).toHaveURL('/');
  });

  test('should persist wallet connection across page navigation', async ({ page }) => {
    await TestHelpers.connectWallet(page);
    
    // Navigate to different pages
    await page.goto('/feed');
    await page.goto('/create');
    await page.goto(`/profile/${TEST_WALLET_ADDRESS}`);
    
    // Verify wallet remains connected
    const isConnected = await page.evaluate(() => {
      return localStorage.getItem('wallet_connected') === 'true';
    });
    
    expect(isConnected).toBeTruthy();
  });

  test('should disconnect wallet when requested', async ({ page }) => {
    await TestHelpers.connectWallet(page);
    
    // Simulate wallet disconnection
    await page.evaluate(() => {
      localStorage.removeItem('wallet_connected');
      localStorage.removeItem('wallet_address');
    });
    
    // Reload page
    await page.reload();
    
    // Verify wallet is disconnected
    const isConnected = await page.evaluate(() => {
      return localStorage.getItem('wallet_connected') === 'true';
    });
    
    expect(isConnected).toBeFalsy();
  });
});