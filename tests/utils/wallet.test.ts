import { expect, test, describe, afterAll, mock } from "bun:test";
import { existsSync, unlinkSync, readFileSync } from "fs";
import { decryptBuffer } from "../../src/utils/crypto.js";

// Variables to control mock prompt behavior in tests
let mockPasswordValue: string | null = "my-password";
let mockConfirmValue: string | null = "my-password";
let mockKeyValue: string | null = "my-value";
let promptCallCount = 0;

// Mock the local prompts utility module
mock.module("../../src/utils/prompts.js", () => {
  return {
    promptPassword: async (query: string) => {
      promptCallCount++;
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

// Mock our custom clipboard utility to isolate clipboard operations in headless testing environments
let clipboardContent = "";
mock.module("../../src/utils/clipboard.js", () => {
  return {
    copyToClipboard: async (val: string) => {
      clipboardContent = val;
    }
  };
});

import {
  ensureWalletExists,
  addWalletKey,
  listWalletKeys,
  copyWalletValue,
  deleteWalletKey
} from "../../src/utils/wallet.js";

describe("Wallet Initialization & Key-Value Utilities", () => {
  const testWalletFile = "temp_test_wallet.enc";

  afterAll(() => {
    try { unlinkSync(testWalletFile); } catch {}
  });

  test("creates wallet.enc if it does not exist", async () => {
    try { unlinkSync(testWalletFile); } catch {}
    
    mockPasswordValue = "supersecret";
    mockConfirmValue = "supersecret";
    promptCallCount = 0;

    const result = await ensureWalletExists(testWalletFile);
    
    expect(result).toBe(true);
    expect(existsSync(testWalletFile)).toBe(true);
    expect(promptCallCount).toBe(2); // One for creation prompt, one for confirmation prompt

    // Verify it was encrypted with the correct password by decrypting it
    expect(() => {
      const encrypted = readFileSync(testWalletFile);
      decryptBuffer(encrypted, "supersecret");
    }).not.toThrow();
  });

  test("returns true immediately if file exists", async () => {
    expect(existsSync(testWalletFile)).toBe(true);
    promptCallCount = 0;

    const result = await ensureWalletExists(testWalletFile);
    
    expect(result).toBe(true);
    expect(promptCallCount).toBe(0); // Should not prompt at all
  });

  test("returns false and cancels if password input is cancelled", async () => {
    const cancelWalletFile = "temp_cancel_wallet.enc";
    try { unlinkSync(cancelWalletFile); } catch {}

    mockPasswordValue = null; // simulate cancellation
    mockConfirmValue = "any";

    const result = await ensureWalletExists(cancelWalletFile);
    
    expect(result).toBe(false);
    expect(existsSync(cancelWalletFile)).toBe(false);
  });

  test("adds a key-value pair successfully to an existing wallet", async () => {
    try { unlinkSync(testWalletFile); } catch {}

    // Initialize the wallet
    mockPasswordValue = "supersecret";
    mockConfirmValue = "supersecret";
    await ensureWalletExists(testWalletFile);

    // Add key-value pair
    mockPasswordValue = "supersecret"; // wallet decryption password
    mockKeyValue = "my-secret-value"; // value for key
    promptCallCount = 0;

    const result = await addWalletKey("mykey", testWalletFile);

    expect(result).toBe(true);

    // Verify the entry was written by decrypting in-memory and checking JSON entries record
    const encrypted = readFileSync(testWalletFile);
    const decrypted = decryptBuffer(encrypted, "supersecret");
    const data = JSON.parse(decrypted.toString("utf-8"));
    expect(data.entries["mykey"]).toBe("my-secret-value");
  });

  test("lists keys in the wallet successfully", async () => {
    mockPasswordValue = "supersecret";
    const result = await listWalletKeys(testWalletFile);
    expect(result).toBe(true);
  });

  test("copies a key value to clipboard successfully", async () => {
    mockPasswordValue = "supersecret";
    clipboardContent = "";

    const result = await copyWalletValue("mykey", testWalletFile);
    expect(result).toBe(true);
    expect(clipboardContent).toBe("my-secret-value");
  });

  test("deletes a key from the wallet successfully", async () => {
    mockPasswordValue = "supersecret";
    
    const result = await deleteWalletKey("mykey", testWalletFile);
    expect(result).toBe(true);

    // Verify key is deleted in-memory
    const encrypted = readFileSync(testWalletFile);
    const decrypted = decryptBuffer(encrypted, "supersecret");
    const data = JSON.parse(decrypted.toString("utf-8"));
    expect(data.entries["mykey"]).toBeUndefined();
  });
});
