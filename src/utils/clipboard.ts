import { spawn } from 'child_process';

/**
 * Copies a string to the system clipboard using native platform commands.
 * Zero external dependencies.
 */
export function copyToClipboard(text: string): Promise<void> {
  return new Promise((resolve, reject) => {
    let command = '';
    let args: string[] = [];

    if (process.platform === 'darwin') {
      command = 'pbcopy';
    } else if (process.platform === 'win32') {
      command = 'clip';
    } else {
      // Linux/BSD standard command
      command = 'xclip';
      args = ['-selection', 'clipboard'];
    }

    const child = spawn(command, args, { stdio: ['pipe', 'ignore', 'ignore'] });

    child.on('error', (err) => {
      // Fallback to xsel on Linux if xclip fails or is not installed
      if (process.platform !== 'darwin' && process.platform !== 'win32' && command === 'xclip') {
        const fallback = spawn('xsel', ['-ib'], { stdio: ['pipe', 'ignore', 'ignore'] });
        fallback.on('error', (fallbackErr) => {
          reject(new Error('Neither xclip nor xsel is installed or working.'));
        });
        fallback.stdin.write(text);
        fallback.stdin.end();
        fallback.on('close', (code) => {
          if (code === 0) resolve();
          else reject(new Error(`xsel exited with code ${code}`));
        });
        return;
      }
      reject(err);
    });

    child.stdin.write(text);
    child.stdin.end();

    child.on('close', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`${command} exited with code ${code}`));
      }
    });
  });
}
