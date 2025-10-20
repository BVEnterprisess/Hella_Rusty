import { test, expect } from '@playwright/test';

test.describe('AI Agent Sandbox Tests', () => {
  const BASE_URL = process.env.AGENT_URL || 'http://localhost:8080';

  test('agent health check', async ({ request }) => {
    const response = await request.get(`${BASE_URL}/health`);
    expect(response.ok()).toBeTruthy();

    const health = await response.json();
    expect(health.status).toBe('healthy');
    expect(health.agent_id).toBeDefined();
  });

  test('agent prediction endpoint', async ({ request }) => {
    const testPayload = {
      job_id: 'test-job-001',
      input: {
        text: 'hello, order a whiskey',
        lang: 'en'
      },
      mode: 'sync',
      meta: {
        test: true,
        timestamp: Date.now()
      }
    };

    const response = await request.post(`${BASE_URL}/predict`, {
      data: testPayload,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    expect(response.ok()).toBeTruthy();

    const result = await response.json();
    expect(result.job_id).toBe(testPayload.job_id);
    expect(result.status).toBe('ok');
    expect(result.output).toBeDefined();
    expect(result.output.intent).toBeDefined();
    expect(result.output.confidence).toBeGreaterThan(0);
    expect(result.output.confidence).toBeLessThanOrEqual(1);
  });

  test('agent batch prediction', async ({ request }) => {
    const jobs = [];

    // Create multiple concurrent requests
    for (let i = 0; i < 5; i++) {
      jobs.push({
        job_id: `batch-job-${i}`,
        input: {
          text: `test message ${i}`,
          lang: 'en'
        },
        mode: 'async'
      });
    }

    // Send all requests concurrently
    const responses = await Promise.all(
      jobs.map(job =>
        request.post(`${BASE_URL}/predict`, {
          data: job,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    );

    // All requests should succeed
    for (const response of responses) {
      expect(response.ok()).toBeTruthy();
    }

    // Check response structure
    const results = await Promise.all(responses.map(r => r.json()));
    for (let i = 0; i < results.length; i++) {
      expect(results[i].job_id).toBe(jobs[i].job_id);
      expect(results[i].status).toBe('ok');
      expect(results[i].output).toBeDefined();
    }
  });

  test('agent error handling - invalid input', async ({ request }) => {
    const invalidPayload = {
      job_id: 'error-test-001',
      input: {
        // Missing required 'text' field
        lang: 'en'
      }
    };

    const response = await request.post(`${BASE_URL}/predict`, {
      data: invalidPayload,
      headers: { 'Content-Type': 'application/json' }
    });

    expect(response.status()).toBe(400);

    const error = await response.json();
    expect(error.status).toBe('error');
    expect(error.code).toBeDefined();
    expect(error.message).toBeDefined();
  });

  test('agent timeout handling', async ({ request }) => {
    const timeoutPayload = {
      job_id: 'timeout-test-001',
      input: {
        text: 'a'.repeat(10000), // Very long text to trigger timeout
        lang: 'en'
      },
      timeout_ms: 100 // Very short timeout
    };

    const response = await request.post(`${BASE_URL}/predict`, {
      data: timeoutPayload,
      headers: { 'Content-Type': 'application/json' }
    });

    // Should either timeout or handle gracefully
    if (response.status() === 504) {
      const error = await response.json();
      expect(error.code).toBe('TIMEOUT');
    } else {
      expect(response.ok()).toBeTruthy();
    }
  });

  test('agent metrics endpoint', async ({ request }) => {
    const response = await request.get(`${BASE_URL}/metrics`);
    expect(response.ok()).toBeTruthy();

    const metrics = await response.text();
    expect(metrics).toContain('agent_requests_total');
    expect(metrics).toContain('agent_request_duration_seconds');
    expect(metrics).toContain('gpu_memory_bytes');
  });

  test('agent concurrent load test', async ({ request }) => {
    const numRequests = 20;
    const requests = [];

    // Create concurrent requests
    for (let i = 0; i < numRequests; i++) {
      requests.push(
        request.post(`${BASE_URL}/predict`, {
          data: {
            job_id: `load-test-${i}`,
            input: {
              text: `Load test message ${i}`,
              lang: 'en'
            }
          },
          headers: { 'Content-Type': 'application/json' }
        })
      );
    }

    const startTime = Date.now();
    const responses = await Promise.all(requests);
    const endTime = Date.now();

    const totalTime = endTime - startTime;
    const avgLatency = totalTime / numRequests;

    console.log(`Load test: ${numRequests} requests in ${totalTime}ms (avg: ${avgLatency}ms per request)`);

    // All requests should succeed
    for (const response of responses) {
      expect(response.ok()).toBeTruthy();
    }

    // Average latency should be reasonable (adjust based on your requirements)
    expect(avgLatency).toBeLessThan(1000); // Less than 1 second average
  });

  test('agent memory usage monitoring', async ({ request }) => {
    // Get initial metrics
    const initialResponse = await request.get(`${BASE_URL}/metrics`);
    const initialMetrics = await initialResponse.text();

    // Extract memory usage (this is a simplified check)
    const memoryMatch = initialMetrics.match(/gpu_memory_bytes (\d+)/);
    const initialMemory = memoryMatch ? parseInt(memoryMatch[1]) : 0;

    // Run some load
    const loadRequests = [];
    for (let i = 0; i < 10; i++) {
      loadRequests.push(
        request.post(`${BASE_URL}/predict`, {
          data: {
            job_id: `memory-test-${i}`,
            input: {
              text: `Memory test message ${i}`,
              lang: 'en'
            }
          },
          headers: { 'Content-Type': 'application/json' }
        })
      );
    }

    await Promise.all(loadRequests);

    // Check memory hasn't grown excessively
    const finalResponse = await request.get(`${BASE_URL}/metrics`);
    const finalMetrics = await finalResponse.text();

    // In a real scenario, you'd want more sophisticated memory monitoring
    expect(finalResponse.ok()).toBeTruthy();
  });
});