export const testSecrets = {
  name: 'TEST_SECRET',
  value: 'test_value_12345',
  description: 'Test secret for E2E tests',
};

export const testApiData = {
 健康: {
   status: 'ok',
 },
  user: {
    id: 1,
    name: 'Test User',
    email: 'test@example.com',
  },
};

export const testLogEntries = [
  '2024-01-15 10:00:00 INFO Application started',
  '2024-01-15 10:00:01 DEBUG Processing request',
  '2024-01-15 10:00:02 WARN Slow query detected',
  '2024-01-15 10:00:03 ERROR Database connection failed',
  '2024-01-15 10:00:04 FATAL System crash',
];

export const testGitCommits = {
  good: 'HEAD~5',
  bad: 'HEAD',
};

export const testPorts = {
  api: 8080,
  test: 9999,
  integration: 9876,
};
