import path from "path"
import os from "os"

const CONFIG_PATH = ".config/wallet.enc"
const SAFE_DIR = os.homedir()

const RESERVED_KEYS = new Set([
  '__proto__',
  'constructor',
  'prototype',
  'toString',
  'valueOf',
  'hasOwnProperty',
  'isPrototypeOf',
  'propertyIsEnumerable',
  'toLocaleString',
])

const VALID_KEY_REGEX = /^[a-zA-Z0-9._-]+$/

export function userPath(filepath: string): string {
  if (filepath === "default") {
    return path.join(SAFE_DIR, CONFIG_PATH)
  }

  const resolved = path.resolve(filepath)

  if (resolved.includes('..') || !resolved.startsWith(SAFE_DIR)) {
    throw new Error(`unsafe path: ${filepath}`)
  }

  return resolved
}

export function validateKey(key: string): void {
  if (RESERVED_KEYS.has(key)) {
    throw new Error(`reserved key: ${key}`)
  }
  if (!VALID_KEY_REGEX.test(key)) {
    throw new Error(`invalid key: ${key}`)
  }
}
