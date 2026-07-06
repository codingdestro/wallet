import readline from 'readline';

/**
 * Prompts the user for a password, masking the input with '*' on keypress.
 * Zero external dependencies.
 * @param query The prompt message to show
 */
export function promptPassword(query: string): Promise<string> {
  return new Promise((resolve) => {
    // Fallback to standard plain text readline in non-TTY (non-interactive) environments
    if (!process.stdin.isTTY) {
      const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout
      });
      rl.question(query, (answer) => {
        rl.close();
        resolve(answer);
      });
      return;
    }

    readline.emitKeypressEvents(process.stdin);
    process.stdin.setRawMode(true);
    process.stdin.resume();
    process.stdout.write(query);

    let password = '';

    const onKeypress = (str: string, key: any) => {
      // Ctrl+C cancellation
      if (key && key.ctrl && key.name === 'c') {
        process.stdout.write('\n');
        process.stdin.setRawMode(false);
        process.stdin.removeListener('keypress', onKeypress);
        process.stdin.pause();
        resolve(null as any);
        return;
      }

      // Enter key submits the input
      if (key && key.name === 'return') {
        process.stdout.write('\n');
        process.stdin.setRawMode(false);
        process.stdin.removeListener('keypress', onKeypress);
        process.stdin.pause();
        resolve(password);
        return;
      }

      // Backspace erases the last character
      if (key && key.name === 'backspace') {
        if (password.length > 0) {
          password = password.slice(0, -1);
          process.stdout.write('\b \b'); // backspace, space, backspace to clear character
        }
      } else if (str && str.length === 1) {
        const code = str.charCodeAt(0);
        // Capture only printable ASCII characters
        if (code >= 32 && code <= 126) {
          password += str;
          process.stdout.write('*');
        }
      }
    };

    process.stdin.on('keypress', onKeypress);
  });
}
