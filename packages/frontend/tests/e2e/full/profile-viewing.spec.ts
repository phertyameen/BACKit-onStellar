import { test, expect } from '@playwright/test';
import { TestHelpers, ApiMocks, TEST_WALLET_ADDRESS } from './utils';

test.describe('Profile Viewing Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Mock API responses
    await ApiMocks.mockUserProfileApi(page, TEST_WALLET_ADDRESS);
    await ApiMocks.mockTokensApi(page);
    
    // Connect wallet
    await TestHelpers.connectWallet(page);
  });

  test('should view user profile successfully', async ({ page }) => {
    // Navigate to profile page
    await page.goto(`/profile/${TEST_WALLET_ADDRESS}`);
    
    // Verify profile page loaded
    await expect(page.getByText('Profile')).toBeVisible();
    
    // Verify user information is displayed
    await expect(page.getByText(TEST_WALLET_ADDRESS.substring(0, 10))).toBeVisible();
    await expect(page.getByText('Test User')).toBeVisible();
    
    // Verify stats are displayed
    await expect(page.getByText('Win Rate')).toBeVisible();
    await expect(page.getByText('Total Calls')).toBeVisible();
    await expect(page.getByText('Followers')).toBeVisible();
    await expect(page.getByText('Following')).toBeVisible();
  });

  test('should display correct profile statistics', async ({ page }) => {
    await page.goto(`/profile/${TEST_WALLET_ADDRESS}`);
    
    // Verify specific stat values (based on mock data)
    await expect(page.getByText('75%')).toBeVisible(); // Win rate
    await expect(page.getByText('12')).toBeVisible(); // Total calls
    await expect(page.getByText('42')).toBeVisible(); // Followers
    await expect(page.getByText('18')).toBeVisible(); // Following
  });

  test('should show different tabs for profile content', async ({ page }) => {
    await page.goto(`/profile/${TEST_WALLET_ADDRESS}`);
    
    // Verify tabs exist
    await expect(page.getByRole('tab', { name: 'Created' })).toBeVisible();
    await expect(page.getByRole('tab', { name: 'Participated' })).toBeVisible();
    await expect(page.getByRole('tab', { name: 'Resolved' })).toBeVisible();
    
    // Click on different tabs
    await page.getByRole('tab', { name: 'Participated' }).click();
    await page.getByRole('tab', { name: 'Resolved' }).click();
    await page.getByRole('tab', { name: 'Created' }).click();
  });

  test('should handle profile not found gracefully', async ({ page }) => {
    // Mock 404 response for non-existent user
    await page.route('/api/users/nonexistent', async (route) => {
      await route.fulfill({
        status: 404,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'User not found' })
      });
    });
    
    await page.goto('/profile/nonexistent');
    
    // Should show error message
    await expect(page.getByText('Profile Not Found')).toBeVisible();
    await expect(page.getByText('The user with address nonexistent was not found')).toBeVisible();
  });

  test('should allow following/unfollowing users', async ({ page }) => {
    await page.goto(`/profile/${TEST_WALLET_ADDRESS}`);
    
    // Initially should show follow button (based on mock data)
    const followButton = page.getByRole('button', { name: 'Follow' });
    await expect(followButton).toBeVisible();
    
    // Click follow
    await followButton.click();
    
    // Should show unfollow button
    const unfollowButton = page.getByRole('button', { name: 'Unfollow' });
    await expect(unfollowButton).toBeVisible();
    
    // Click unfollow
    await unfollowButton.click();
    
    // Should show follow button again
    await expect(followButton).toBeVisible();
  });

  test('should show user\'s created calls', async ({ page }) => {
    // Mock profile with created calls
    await page.route(`/api/users/${TEST_WALLET_ADDRESS}`, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          user: {
            address: TEST_WALLET_ADDRESS,
            displayName: 'Test User',
            winRate: 75,
            totalCalls: 12,
            followers: 42,
            following: 18,
            isFollowing: false
          },
          createdCalls: [
            {
              id: 'call-1',
              title: 'BTC Prediction',
              token: 'BTC',
              stake: 100,
              createdAt: Date.now()
            }
          ],
          participatedCalls: [],
          resolvedCalls: []
        })
      });
    });
    
    await page.goto(`/profile/${TEST_WALLET_ADDRESS}`);
    
    // Switch to Created tab
    await page.getByRole('tab', { name: 'Created' }).click();
    
    // Verify created calls are displayed
    await expect(page.getByText('BTC Prediction')).toBeVisible();
  });

  test('should maintain profile state during navigation', async ({ page }) => {
    await page.goto(`/profile/${TEST_WALLET_ADDRESS}`);
    
    // Store initial state
    const initialDisplayName = await page.getByText('Test User').textContent();
    
    // Navigate away and back
    await page.goto('/feed');
    await page.goto(`/profile/${TEST_WALLET_ADDRESS}`);
    
    // Verify state is maintained
    const currentDisplayName = await page.getByText('Test User').textContent();
    expect(currentDisplayName).toEqual(initialDisplayName);
  });
});