import path from "path"
import fs from "fs"
import os from "os"


const CONFIG_PATH = ".config/wallet.enc"

export function userPath(filepath: string): (string) {
    return filepath === "default" ? path.join(os.homedir(), CONFIG_PATH) : filepath
}
