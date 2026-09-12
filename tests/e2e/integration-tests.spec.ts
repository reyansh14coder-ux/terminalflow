import { test, expect } from '@playwright/test';
import { exec } from 'child_process';
import { promisify } from 'util';
import * as fs from 'fs';
import * as path from 'path';

const execAsync = promisify(exec);

test.describe('Integration Tests', () => {
  test.describe('Secret Management', () => {
    const testSecretName = 'INTEGRATION_TEST_SECRET';
    const testSecretValue = 'integration_test_value_12345';

    test.afterAll(async () => {
      try {
        await execAsync(`cargo run -- secret delete ${testSecretName}`, { timeout: 10000 });
      } catch {}
    });

    test('should create a secret', async () => {
      const { stdout } = await execAsync(
        `cargo run -- secret set ${testSecretName} "${testSecretValue}"`,
        { timeout: 10000 }
      );
      expect(stdout).toContain('✅');
    });

    test('should retrieve a secret', async () => {
      const { stdout } = await execAsync(
        `cargo run -- secret get ${testSecretName}`,
        { timeout: 10000 }
      );
      expect(stdout).toContain(testSecretValue);
    });

    test('should list secrets including the new one', async () => {
      const { stdout } = await execAsync('cargo run -- secret list', { timeout: 10000 });
      expect(stdout).toContain(testSecretName);
    });

    test('should delete a secret', async () => {
      const { stdout } = await execAsync(
        `cargo run -- secret delete ${testSecretName}`,
        { timeout: 10000 }
      );
      expect(stdout).toContain('✅');
    });

    test('should not find deleted secret', async () => {
      const { stdout } = await execAsync(
        `cargo run -- secret get ${testSecretName}`,
        { timeout: 10000 }
      );
      expect(stdout).toContain('not found');
    });
  });

  test.describe('Code Generation', () => {
    test('should generate a struct', async () => {
      const { stdout } = await execAsync(
        'cargo run -- gen struct Product',
        { timeout: 10000 }
      );
      expect(stdout).toContain('pub struct Product');
      expect(stdout).toContain('impl Product');
    });

    test('should generate an API endpoint', async () => {
      const { stdout } = await execAsync(
        'cargo run -- gen endpoint POST /api/users',
        { timeout: 10000 }
      );
      expect(stdout).toContain('pub async fn');
    });

    test('should generate tests', async () => {
      const { stdout } = await execAsync(
        'cargo run -- gen tests my_function',
        { timeout: 10000 }
      );
      expect(stdout).toContain('#[cfg(test)]');
      expect(stdout).toContain('mod tests');
    });
  });

  test.describe('File Operations', () => {
    const testDir = '/tmp/terminalflow_test';
    const testFile = path.join(testDir, 'test.txt');

    test.beforeAll(async () => {
      await execAsync(`mkdir -p ${testDir}`);
    });

    test.afterAll(async () => {
      await execAsync(`rm -rf ${testDir}`);
    });

    test('should watch file for changes', async () => {
      // Create test file
      fs.writeFileSync(testFile, 'initial content');
      
      // Start watcher
      const child = exec(`cargo run -- watch ${testFile}`);
      
      // Wait for watcher to start
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Modify file
      fs.writeFileSync(testFile, 'modified content');
      
      // Wait for detection
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Cleanup
      child.kill();
    });
  });

  test.describe('Log Analysis', () => {
    const testLogFile = '/tmp/test_integration.log';

    test.beforeAll(async () => {
      const logContent = `
2024-01-15 10:00:00 INFO Application started
2024-01-15 10:00:01 DEBUG Processing request
2024-01-15 10:00:02 WARN Slow query detected
2024-01-15 10:00:03 ERROR Database connection failed
2024-01-15 10:00:04 FATAL System crash
`.trim();
      fs.writeFileSync(testLogFile, logContent);
    });

    test.afterAll(async () => {
      fs.unlinkSync(testLogFile);
    });

    test('should parse and display logs', async () => {
      const { stdout } = await execAsync(
        `cargo run -- logs ${testLogFile}`,
        { timeout: 10000 }
      );
      expect(stdout).toContain('Application started');
      expect(stdout).toContain('Database connection failed');
    });

    test('should filter logs by level', async () => {
      const { stdout } = await execAsync(
        `cargo run -- logs ${testLogFile} --level error`,
        { timeout: 10000 }
      );
      expect(stdout).toContain('ERROR');
    });
  });

  test.describe('Process Management', () => {
    test('should list processes', async () => {
      const { stdout } = await execAsync('cargo run -- ps', { timeout: 10000 });
      expect(stdout).toContain('Processes');
    });

    test('should filter processes', async () => {
      const { stdout } = await execAsync(
        'cargo run -- ps --filter cargo',
        { timeout: 10000 }
      );
      expect(stdout).toContain('cargo');
    });

    test('should show top CPU processes', async () => {
      const { stdout } = await execAsync(
        'cargo run -- ps --top-cpu',
        { timeout: 10000 }
      );
      expect(stdout).toContain('CPU');
    });
  });

  test.describe('API Server Integration', () => {
    let serverProcess: any;
    const testPort = 9876;

    test.beforeAll(async () => {
      serverProcess = exec(`cargo run -- serve --port ${testPort}`);
      await new Promise(resolve => setTimeout(resolve, 2000));
    });

    test.afterAll(async () => {
      if (serverProcess) {
        serverProcess.kill();
      }
    });

    test('should start server and respond to health check', async () => {
      const response = await fetch(`http://localhost:${testPort}/health`);
      expect(response.ok).toBeTruthy();
      
      const data = await response.json();
      expect(data.status).toBe('ok');
    });

    test('should handle multiple requests', async () => {
      const requests = Array(5).fill(null).map((_, i) => 
        fetch(`http://localhost:${testPort}/health`)
      );
      
      const responses = await Promise.all(requests);
      responses.forEach(response => {
        expect(response.ok).toBeTruthy();
      });
    });
  });

  test.describe('Git Integration', () => {
    test('should show git status', async () => {
      const { stdout } = await execAsync('cargo run -- status', { timeout: 10000 });
      expect(stdout).toContain('Git Status');
      expect(stdout).toContain('Branch');
    });

    test('should handle git bisect command', async () => {
      const { stdout, stderr } = await execAsync(
        'cargo run -- bisect start HEAD~1 HEAD',
        { timeout: 10000 }
      );
      const output = stdout + stderr;
      expect(output).toContain('bisect');
    });
  });
});
