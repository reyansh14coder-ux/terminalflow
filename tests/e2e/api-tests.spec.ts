import { test, expect } from '@playwright/test';

test.describe('API Server', () => {
  test('should respond to health check', async ({ request }) => {
    const response = await request.get('/health');
    expect(response.ok()).toBeTruthy();
    
    const body = await response.json();
    expect(body.status).toBe('ok');
  });

  test('should return 404 for unknown routes', async ({ request }) => {
    const response = await request.get('/nonexistent');
    expect(response.status()).toBe(404);
  });

  test('should handle POST requests', async ({ request }) => {
    const response = await request.post('/api/data', {
      data: { key: 'value' },
    });
    
    expect(response.status()).toBeLessThan(500);
  });

  test('should handle query parameters', async ({ request }) => {
    const response = await request.get('/api/search?q=test&page=1');
    expect(response.status()).toBeLessThan(500);
  });

  test('should handle headers', async ({ request }) => {
    const response = await request.get('/health', {
      headers: {
        'X-Custom-Header': 'test-value',
      },
    });
    expect(response.ok()).toBeTruthy();
  });

  test('should handle large payloads', async ({ request }) => {
    const largeData = {
      data: Array(1000).fill({ item: 'test', value: 123 }),
    };
    
    const response = await request.post('/api/data', {
      data: largeData,
    });
    
    expect(response.status()).toBeLessThan(500);
  });

  test('should respond within timeout', async ({ request }) => {
    const start = Date.now();
    const response = await request.get('/health');
    const duration = Date.now() - start;
    
    expect(response.ok()).toBeTruthy();
    expect(duration).toBeLessThan(5000);
  });

  test('should handle concurrent requests', async ({ request }) => {
    const requests = Array(10).fill(null).map((_, i) => 
      request.get(`/api/data?id=${i}`)
    );
    
    const responses = await Promise.all(requests);
    responses.forEach(response => {
      expect(response.status()).toBeLessThan(500);
    });
  });
});
