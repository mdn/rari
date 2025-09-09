#!/usr/bin/env node

import { spawn } from "node:child_process";
import process from "node:process";
import { rariBin } from "./index.js";

const input = process.argv.slice(2);

spawn(rariBin, input, { stdio: "inherit" }).on("exit", (code, signal) => {
  if (signal) {
    try {
      process.kill(process.pid, signal);
    } catch {
      // Reflect signal code in exit code.
      // See: https://nodejs.org/api/os.html#os-constants
      const signalCode = os.constants?.signals?.[signal] ?? 0;
      process.exit(128 + signalCode);
    }
  }
  process.exit(code ?? 0);
});
