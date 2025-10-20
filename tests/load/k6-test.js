import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const responseTimeTrend = new Trend('response_time');
const errorRate = new Rate('error_rate');

// Test configuration
export const options = {
  stages: [
    { duration: '2m', target: 10 },   // Ramp up to 10 users over 2 minutes
    { duration: '5m', target: 50 },   // Ramp up to 50 users over 5 minutes
    { duration: '10m', target: 100 }, // Ramp up to 100 users over 10 minutes
    { duration: '5m', target: 50 },   // Ramp down to 50 users
    { duration: '2m', target: 0 },    // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests should be below 500ms
    error_rate: ['rate<0.1'],         // Error rate should be below 10%
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8000';

// Test scenarios
export default function () {
  // Test basic agent query
  const queryPayload = JSON.stringify({
    message: 'What is the capital of France?',
    agent_type: 'general',
    max_tokens: 150,
    temperature: 0.7
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  const response = http.post(`${BASE_URL}/api/v1/query`, queryPayload, params);

  // Record metrics
  responseTimeTrend.add(response.timings.duration);
  errorRate.add(response.status !== 200);

  // Validate response
  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 1000ms': (r) => r.timings.duration < 1000,
    'response has content': (r) => r.body && r.body.length > 0,
  });

  sleep(1);
}

// Setup function - runs before the test
export function setup() {
  console.log('Setting up load test...');

  // Health check
  const healthResponse = http.get(`${BASE_URL}/health`);
  if (healthResponse.status !== 200) {
    console.error('Health check failed!');
    return;
  }

  console.log('Load test setup complete');
  return { timestamp: new Date().toISOString() };
}

// Teardown function - runs after the test
export function teardown(data) {
  console.log('Load test completed at:', data.timestamp);
}

// Handle summary - custom summary output
export function handleSummary(data) {
  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    './tests/load/k6-results.json': JSON.stringify(data, null, 2),
  };
}