import { expect, test, describe, afterAll, mock } from "bun:test";
import { existsSync, unlinkSync, readFileSync } from "fs";
import { decryptFile } from "../../src/utils/crypto.js";

// Variables to control mock prompt behavior in tests
let mockPasswordValue: string | null = "my-password";
let mockConfirmValue: string | null = "my-password";
let mockKeyValue: string | null = "my-value";
let promptCallCount = 0;

// Use mock.module in Bun to mock the external package @clack/prompts
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
      promptCallCount++;
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

import { ensureWalletExists, addWalletKey } from "../../src/utils/wallet.js";

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
    const decryptedTemp = "temp_decrypted_wallet.json";
    expect(() => {
      decryptFile(testWalletFile, decryptedTemp, "supersecret");
    }).not.toThrow();
    try { unlinkSync(decryptedTemp); } catch {}
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

    // Verify the entry was written by decrypting and checking JSON entries record
    const decryptedTemp = "temp_dec_verify.json";
    decryptFile(testWalletFile, decryptedTemp, "supersecret");
    const rawContent = readFileSync(decryptedTemp, "utf-8");
    const data = JSON.parse(rawContent);
    expect(data.entries["mykey"]).toBe("my-secret-value");

    unlinkSync(decryptedTemp);
  });
});
