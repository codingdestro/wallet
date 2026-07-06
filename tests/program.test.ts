import { expect, test, describe, afterAll, mock } from "bun:test";
import { writeFileSync, readFileSync, unlinkSync, existsSync } from "fs";
import { decryptFile } from "../src/utils/crypto.js";

// Control prompt inputs
let mockPasswordValue: string | null = "my-password";
let mockConfirmValue: string | null = "my-password";
let mockKeyValue: string | null = "my-value";

mock.module("@clack/prompts", () => {
  return {
    intro: () => {},
    outro: () => {},
    cancel: () => {},
    isCancel: (val: any) => val === null,
    log: {
      error: () => {},
    },
    spinner: () => ({
      start: () => {},
      stop: () => {},
    }),
    password: async ({ message }: any) => {
      if (message.includes("Confirm")) {
        return mockConfirmValue;
      }
      if (message.includes("value for key")) {
        return mockKeyValue;
      }
      return mockPasswordValue;
    },
  };
});

import { createProgram } from "../src/program.js";

describe("CLI program integration", () => {
  const testWallet = "temp_cli_wallet.enc";

  afterAll(() => {
    try { unlinkSync(testWallet); } catch {}
  });

  test("runs init and add commands via createProgram parse", async () => {
    try { unlinkSync(testWallet); } catch {}

    const program = createProgram();

    // 1. Run init command
    mockPasswordValue = "supersecret";
    mockConfirmValue = "supersecret";
    await program.parseAsync(["node", "index.js", "init", "-f", testWallet]);

    expect(existsSync(testWallet)).toBe(true);

    // 2. Run add command
    mockPasswordValue = "supersecret"; // decryption password
    mockKeyValue = "my-secret-key-value"; // value to store
    
    const programAdd = createProgram();
    await programAdd.parseAsync(["node", "index.js", "add", "mykey", "-f", testWallet]);

    // 3. Verify content
    const decryptedTemp = "temp_cli_dec.json";
    decryptFile(testWallet, decryptedTemp, "supersecret");
    const data = JSON.parse(readFileSync(decryptedTemp, "utf-8"));
    expect(data.entries["mykey"]).toBe("my-secret-key-value");

    unlinkSync(decryptedTemp);
  });
});
