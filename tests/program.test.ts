import { expect, test, describe, afterAll, mock } from "bun:test";
import { writeFileSync, readFileSync, unlinkSync, existsSync } from "fs";
import { decryptBuffer } from "../src/utils/crypto.js";

// Control prompt inputs
let mockPasswordValue: string | null = "my-password";
let mockConfirmValue: string | null = "my-password";
let mockKeyValue: string | null = "my-value";
let mockNewPasswordValue: string | null = "new-password";

mock.module("../src/utils/prompts.js", () => {
  return {
    promptPassword: async (query: string) => {
      if (query.includes("confirm")) {
        return mockConfirmValue;
      }
      if (query.includes("value for")) {
        return mockKeyValue;
      }
      if (query.includes("new password")) {
        return mockNewPasswordValue;
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
import { userPath } from "../src/utils/userpath.js";

describe("CLI program integration", () => {
  const testWallet = userPath("default");

  afterAll(() => {
    try { unlinkSync(testWallet); } catch { }
  });

  test("runs full CLI commands lifecycle (init, add, list, copy, delete)", async () => {
    try { unlinkSync(testWallet); } catch { }

    const program = createProgram();

    // 1. Init command
    mockPasswordValue = "supersecret12";
    mockConfirmValue = "supersecret12";
    await program.parseAsync(["node", "index.js", "init", "-f", testWallet]);
    expect(existsSync(testWallet)).toBe(true);

    // 2. Add command
    mockPasswordValue = "supersecret12";
    mockKeyValue = "my-secret-key-value";
    const programAdd = createProgram();
    await programAdd.parseAsync(["node", "index.js", "add", "mykey", "-f", testWallet]);

    // 3. List command
    mockPasswordValue = "supersecret12";
    const programList = createProgram();
    await programList.parseAsync(["node", "index.js", "list", "-f", testWallet]);

    // 4. Copy command
    mockPasswordValue = "supersecret12";
    clipboardContent = "";
    const programCopy = createProgram();
    await programCopy.parseAsync(["node", "index.js", "copy", "mykey", "-f", testWallet]);
    expect(clipboardContent).toBe("my-secret-key-value");

    // 5. Delete command
    mockPasswordValue = "supersecret12";
    const programDelete = createProgram();
    await programDelete.parseAsync(["node", "index.js", "delete", "mykey", "-f", testWallet]);

    // Verify it is gone in-memory
    const encrypted = readFileSync(testWallet);
    const decrypted = decryptBuffer(encrypted, Buffer.from("supersecret12", "utf-8"));
    const data = JSON.parse(decrypted.toString("utf-8"));
    expect(data.entries["mykey"]).toBeUndefined();
  });

  test("runs passwd command to change the wallet password", async () => {
    try { unlinkSync(testWallet); } catch { }

    // Init and add an entry with the old password
    mockPasswordValue = "supersecret12";
    mockConfirmValue = "supersecret12";
    await createProgram().parseAsync(["node", "index.js", "init", "-f", testWallet]);

    mockKeyValue = "my-secret-key-value";
    await createProgram().parseAsync(["node", "index.js", "add", "mykey", "-f", testWallet]);

    // Change the password
    mockPasswordValue = "supersecret12";
    mockNewPasswordValue = "brandnewpass99";
    mockConfirmValue = "brandnewpass99";
    await createProgram().parseAsync(["node", "index.js", "passwd", "-f", testWallet]);

    // Old password no longer decrypts the wallet
    const encrypted = readFileSync(testWallet);
    expect(() => {
      decryptBuffer(encrypted, Buffer.from("supersecret12", "utf-8"));
    }).toThrow();

    // New password decrypts and entries are preserved
    const decrypted = decryptBuffer(encrypted, Buffer.from("brandnewpass99", "utf-8"));
    const data = JSON.parse(decrypted.toString("utf-8"));
    expect(data.entries["mykey"]).toBe("my-secret-key-value");
  });
});
