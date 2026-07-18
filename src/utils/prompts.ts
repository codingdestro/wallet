import readline from 'readline';

export function promptPassword(query: string): Promise<string> {
  return new Promise((resolve) => {
    if (!process.stdin.isTTY) {
      const nonInteractive = process.env.WALLET_NONINTERACTIVE;
      if (!nonInteractive) {
        process.stderr.write('non-interactive. set WALLET_NONINTERACTIVE=1 to bypass\n');
        process.exit(1);
      }
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

    const cleanup = () => {
      try {
        process.stdin.setRawMode(false);
        process.stdin.pause();
      } catch {}
    };

    const onKeypress = (str: string, key: any) => {
      if (key && key.ctrl && key.name === 'c') {
        process.stdout.write('\n');
        process.stdin.removeListener('keypress', onKeypress);
        cleanup();
        resolve(null as any);
        return;
      }

      if (key && key.name === 'return') {
        process.stdout.write('\n');
        process.stdin.removeListener('keypress', onKeypress);
        cleanup();
        resolve(password);
        return;
      }

      if (key && key.name === 'backspace') {
        if (password.length > 0) {
          password = password.slice(0, -1);
          process.stdout.write('\b \b');
        }
      } else if (str && str.length === 1) {
        const code = str.charCodeAt(0);
        if (code >= 32 && code <= 126) {
          password += str;
          process.stdout.write('*');
        }
      }
    };

    process.stdin.on('keypress', onKeypress);
  });
}
