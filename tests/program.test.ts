import { expect, test, describe, afterAll, mock } from "bun:test";
import { writeFileSync, readFileSync, unlinkSync, existsSync } from "fs";
import { decryptBuffer } from "../src/utils/crypto.js";

// Control prompt inputs
let mockPasswordValue: string | null = "my-password";
let mockConfirmValue: string | null = "my-password";
let mockKeyValue: string | null = "my-value";

mock.module("../src/utils/prompts.js", () => {
  return {
    promptPassword: async (query: string) => {
      if (query.includes("Confirm")) {
        return mockConfirmValue;
      }
      if (query.includes("value for key")) {
        return mockKeyValue;
      }
      return mockPasswordValue;
    }
  };
});

let clipboardContent = "";
mock.module("../src/utils/clipboard.js", () => {
  return {
    copyToClipboard: async (val: string) => {
      clipboardContent = val;
    }
  };
});

import { createProgram } from "../src/program.js";

describe("CLI program integration", () => {
  const testWallet = "temp_cli_wallet.enc";

  afterAll(() => {
    try { unlinkSync(testWallet); } catch {}
  });

  test("runs full CLI commands lifecycle (init, add, list, copy, delete)", async () => {
    try { unlinkSync(testWallet); } catch {}

    const program = createProgram();

    // 1. Init command
    mockPasswordValue = "supersecret";
    mockConfirmValue = "supersecret";
    await program.parseAsync(["node", "index.js", "init", "-f", testWallet]);
    expect(existsSync(testWallet)).toBe(true);

    // 2. Add command
    mockPasswordValue = "supersecret";
    mockKeyValue = "my-secret-key-value";
    const programAdd = createProgram();
    await programAdd.parseAsync(["node", "index.js", "add", "mykey", "-f", testWallet]);

    // 3. List command
    mockPasswordValue = "supersecret";
    const programList = createProgram();
    await programList.parseAsync(["node", "index.js", "list", "-f", testWallet]);

    // 4. Copy command
    mockPasswordValue = "supersecret";
    clipboardContent = "";
    const programCopy = createProgram();
    await programCopy.parseAsync(["node", "index.js", "copy", "mykey", "-f", testWallet]);
    expect(clipboardContent).toBe("my-secret-key-value");

    // 5. Delete command
    mockPasswordValue = "supersecret";
    const programDelete = createProgram();
    await programDelete.parseAsync(["node", "index.js", "delete", "mykey", "-f", testWallet]);

    // Verify it is gone in-memory
    const encrypted = readFileSync(testWallet);
    const decrypted = decryptBuffer(encrypted, "supersecret");
    const data = JSON.parse(decrypted.toString("utf-8"));
    expect(data.entries["mykey"]).toBeUndefined();
  });
});
