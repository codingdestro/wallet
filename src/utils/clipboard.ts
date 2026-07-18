import { spawn } from 'child_process';

const CLIPBOARD_TIMEOUT_MS = 30_000;

function clearPlatformClipboard(): void {
  let command = '';
  let args: string[] = [];

  if (process.platform === 'darwin') {
    command = 'pbcopy';
  } else if (process.platform === 'win32') {
    command = 'clip';
  } else {
    command = 'xclip';
    args = ['-selection', 'clipboard'];
  }

  const child = spawn(command, args, { stdio: ['pipe', 'ignore', 'ignore'] });
  child.stdin.end();
}

export function copyToClipboard(text: string): Promise<void> {
  return new Promise((resolve, reject) => {
    let command = '';
    let args: string[] = [];

    if (process.platform === 'darwin') {
      command = 'pbcopy';
    } else if (process.platform === 'win32') {
      command = 'clip';
    } else {
      command = 'xclip';
      args = ['-selection', 'clipboard'];
    }

    const child = spawn(command, args, { stdio: ['pipe', 'ignore', 'ignore'] });

    child.on('error', (err) => {
      if (process.platform !== 'darwin' && process.platform !== 'win32' && command === 'xclip') {
        const fallback = spawn('xsel', ['-ib'], { stdio: ['pipe', 'ignore', 'ignore'] });
        fallback.on('error', (fallbackErr) => {
          reject(new Error('Neither xclip nor xsel is installed or working.'));
        });
        fallback.stdin.write(text);
        fallback.stdin.end();
        fallback.on('close', (code) => {
          if (code === 0) {
            setTimeout(clearPlatformClipboard, CLIPBOARD_TIMEOUT_MS);
            resolve();
          } else reject(new Error(`xsel exited with code ${code}`));
        });
        return;
      }
      reject(err);
    });

    child.stdin.write(text);
    child.stdin.end();

    child.on('close', (code) => {
      if (code === 0) {
        setTimeout(clearPlatformClipboard, CLIPBOARD_TIMEOUT_MS);
        resolve();
      } else {
        reject(new Error(`${command} exited with code ${code}`));
      }
    });
  });
}
