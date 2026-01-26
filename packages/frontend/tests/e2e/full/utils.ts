import { Page, expect, Route } from '@playwright/test';

// Mock Stellar wallet address for testing
export const TEST_WALLET_ADDRESS = 'GD5DQ6KQZYZ2JY5YKZ7XQYBZQZQZQZQZQZQZQZQZQZQZQZQZQZQZQ';

// Test data for calls
export const TEST_CALL_DATA = {
  token: 'USDC',
  condition: 'BTC > $50000 by Jan 2026',
  stake: 100,
  endTime: '2026-01-30T12:00',
  thesis: 'Bitcoin will reach $50,000 by January 2026 based on institutional adoption trends.'
};

// Utility functions for common test actions
export class TestHelpers {
  static async connectWallet(page: Page) {
    // Mock wallet connection - in a real scenario, you'd interact with Freighter
    // For now, we'll simulate the connection by setting up localStorage or cookies
    await page.addInitScript(() => {
      window.localStorage.setItem('wallet_connected', 'true');
      window.localStorage.setItem('wallet_address', 'GD5DQ6KQZYZ2JY5YKZ7XQYBZQZQZQZQZQZQZQZQZQZQZQZQZQZQZQ');
    });
    
    // Reload to apply the mock wallet state
    await page.reload();
  }

  static async createTestCall(page: Page) {
    // Navigate to create call page
    await page.goto('/create');
    
    // Fill in call creation form
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
    
    // Return the call ID from URL
    const url = page.url();
    const callId = url.split('/').pop();
    return callId;
  }

  static async stakeOnCall(page: Page, callId: string, side: 'YES' | 'NO' = 'YES', amount: number = 50) {
    // Navigate to call detail page
    await page.goto(`/call/${callId}`);
    
    // Select stake side
    await page.getByRole('button', { name: side }).click();
    
    // Enter stake amount
    await page.getByPlaceholder('Amount (USDC)').fill(amount.toString());
    
    // Submit stake
    await page.getByRole('button', { name: 'Stake' }).click();
    
    // Wait for confirmation (this would normally involve wallet signing)
    await page.waitForTimeout(2000); // Simulate transaction processing
    
    // Verify stake was recorded
    await expect(page.getByText(`Staked ${amount} USDC on ${side}`)).toBeVisible();
  }

  static async viewProfile(page: Page, address: string = TEST_WALLET_ADDRESS) {
    await page.goto(`/profile/${address}`);
    
    // Verify profile page loaded
    await expect(page.getByText('Profile')).toBeVisible();
    await expect(page.getByText(address.substring(0, 10))).toBeVisible();
  }

  static async navigateToFeed(page: Page) {
    await page.goto('/feed');
    
    // Verify feed loaded
    await expect(page.getByText('For You')).toBeVisible();
    await expect(page.getByText('Following')).toBeVisible();
  }
}

// Mock API responses for consistent testing
export class ApiMocks {
  static async mockTokensApi(page: Page) {
    await page.route('/api/tokens*', async (route: Route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify([
          { symbol: 'USDC', name: 'USD Coin', price: 1.00 },
          { symbol: 'BTC', name: 'Bitcoin', price: 43000.00 },
          { symbol: 'ETH', name: 'Ethereum', price: 2500.00 }
        ])
      });
    });
  }

  static async mockFeedApi(page: Page) {
    await page.route('/api/feed*', async (route: Route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          calls: [
            {
              id: '1',
              creator: TEST_WALLET_ADDRESS,
              title: 'BTC > $50K by Jan 2026',
              description: 'Prediction about Bitcoin price',
              token: 'BTC',
              condition: 'BTC > $50000 by Jan 2026',
              stake: 100,
              startTs: Date.now(),
              endTs: Date.now() + 86400000,
              outcome: 'PENDING',
              participants: 5,
              totalStake: 500
            }
          ]
        })
      });
    });
  }

  static async mockUserProfileApi(page: Page, address: string = TEST_WALLET_ADDRESS) {
    await page.route(`/api/users/${address}`, async (route: Route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          user: {
            address: address,
            displayName: 'Test User',
            winRate: 75,
            totalCalls: 12,
            followers: 42,
            following: 18,
            isFollowing: false
          },
          createdCalls: [],
          participatedCalls: [],
          resolvedCalls: []
        })
      });
    });
  }

  static async mockCallDetailApi(page: Page, callId: string) {
    await page.route(`/api/calls/${callId}`, async (route: Route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          id: callId,
          creator: TEST_WALLET_ADDRESS,
          token: { symbol: 'BTC', price: 43000 },
          condition: 'BTC > $50000 by Jan 2026',
          stake: 100,
          startTime: Date.now(),
          endTime: Date.now() + 86400000,
          thesis: 'Bitcoin will reach $50,000 by January 2026',
          stakes: { yes: 300, no: 150 },
          participants: 8,
          resolved: false
        })
      });
    });
  }

  static async mockStakeApi(page: Page, callId: string) {
    await page.route(`/api/calls/${callId}/stake`, async (route: Route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          xdr: 'AAAA...', // Mock XDR for transaction
          success: true
        })
      });
    });
  }
}