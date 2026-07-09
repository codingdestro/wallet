import pc from 'picocolors';

export type LogLevel = 'silent' | 'error' | 'info' | 'verbose';

const LEVELS: Record<LogLevel, number> = { silent: 0, error: 1, info: 2, verbose: 3 };

let currentLevel: LogLevel =
  (process.env.WALLET_LOG_LEVEL as LogLevel | undefined) ?? 'info';

export function setLogLevel(level: LogLevel) {
  currentLevel = level;
}

const atLeast = (level: LogLevel) => LEVELS[currentLevel] >= LEVELS[level];

export const logger = {
  error: (msg: string) => {
    if (atLeast('error')) console.error(pc.red(msg));
  },
  warn: (msg: string) => {
    if (atLeast('info')) console.log(pc.yellow(msg));
  },
  success: (msg: string) => {
    if (atLeast('info')) console.log(pc.green(msg));
  },
  status: (msg: string) => {
    if (atLeast('verbose')) console.log(pc.yellow(msg));
  },
  header: (msg: string) => {
    if (atLeast('verbose')) console.log(pc.bold(pc.cyan(msg)));
  },
  item: (key: string) => {
    if (atLeast('info')) console.log('-', key);
  },
};
