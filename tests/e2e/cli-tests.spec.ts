import { test, expect } from '@playwright/test';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

test.describe('CLI Commands', () => {
  test('should show version', async () => {
    const { stdout } = await execAsync('cargo run -- --version');
    expect(stdout).toContain('terminalflow');
    expect(stdout).toContain('2.0.0');
  });

  test('should show help', async () => {
    const { stdout } = await execAsync('cargo run -- --help');
    expect(stdout).toContain('AI-Powered Terminal Dashboard');
    expect(stdout).toContain('Commands:');
  });

  test('should show git status', async () => {
    const { stdout } = await execAsync('cargo run -- status');
    expect(stdout).toContain('Git Status');
  });

  test('should show docker status', async () => {
    const { stdout, stderr } = await execAsync('cargo run -- docker', { timeout: 10000 });
    const output = stdout + stderr;
    expect(output).toContain('Docker');
  });

  test('should show process list', async () => {
    const { stdout } = await execAsync('cargo run -- ps');
    expect(stdout).toContain('Processes');
  });

  test('should show system monitor', async () => {
    const { stdout } = await execAsync('cargo run -- monitor');
    expect(stdout).toContain('System Monitor');
  });

  test('should handle invalid commands gracefully', async () => {
    try {
      await execAsync('cargo run -- invalidcommand', { timeout: 10000 });
    } catch (error: any) {
      expect(error.code).not.toBe(137); // Not killed
    }
  });

  test('should set and get secrets', async () => {
    const { stdout: setOutput } = await execAsync(
      'cargo run -- secret set TEST_SECRET "test_value_123"',
      { timeout: 10000 }
    );
    expect(setOutput).toContain('✅');

    const { stdout: getOutput } = await execAsync(
      'cargo run -- secret get TEST_SECRET',
      { timeout: 10000 }
    );
    expect(getOutput).toContain('test_value_123');
  });

  test('should list secrets', async () => {
    const { stdout } = await execAsync('cargo run -- secret list', { timeout: 10000 });
    expect(stdout).toContain('Secret');
  });

  test('should generate code', async () => {
    const { stdout } = await execAsync(
      'cargo run -- gen struct User',
      { timeout: 10000 }
    );
    expect(stdout).toContain('pub struct User');
  });

  test('should handle file watcher', async () => {
    // This test creates a temp file and watches it
    const testFile = '/tmp/test_watch_file.txt';
    await execAsync(`touch ${testFile}`);
    
    // Start watcher in background
    const child = exec(`cargo run -- watch ${testFile}`);
    
    // Wait a bit then modify file
    await new Promise(resolve => setTimeout(resolve, 1000));
    await execAsync(`echo "modified" >> ${testFile}`);
    
    // Wait for watcher to detect
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Cleanup
    child.kill();
    await execAsync(`rm ${testFile}`);
  });

  test('should handle log viewer', async () => {
    // Create a test log file
    const testLog = '/tmp/test.log';
    await execAsync(`echo "2024-01-15 INFO Test message" > ${testLog}`);
    await execAsync(`echo "2024-01-15 ERROR Error message" >> ${testLog}`);
    
    const { stdout } = await execAsync(
      `cargo run -- logs ${testLog} --tail 5`,
      { timeout: 10000 }
    );
    
    expect(stdout).toContain('Test message');
    expect(stdout).toContain('Error message');
    
    // Cleanup
    await execAsync(`rm ${testLog}`);
  });

  test('should handle HTTP client', async () => {
    const { stdout } = await execAsync(
      'cargo run -- http GET https://httpbin.org/get',
      { timeout: 15000 }
    );
    expect(stdout).toContain('200');
  });

  test('should handle git bisect', async () => {
    const { stdout, stderr } = await execAsync(
      'cargo run -- bisect start HEAD~5 HEAD',
      { timeout: 10000 }
    );
    const output = stdout + stderr;
    expect(output).toContain('bisect');
  });

  test('should handle API server startup', async () => {
    // Start server in background
    const child = exec('cargo run -- serve --port 9999');
    
    // Wait for server to start
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Make a request
    try {
      const response = await fetch('http://localhost:9999/health');
      expect(response.ok).toBeTruthy();
    } finally {
      child.kill();
    }
  });
});
