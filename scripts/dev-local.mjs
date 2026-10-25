#!/usr/bin/env node
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");
const gpuiDir = path.join(repoRoot, "apps", "desktop-gpui");
const pidFile = path.join(gpuiDir, "target", "sricap-rec.pid");

const platform = os.platform(); // 'darwin', 'win32', 'linux'
const arch = os.arch();         // 'arm64', 'x64'
const isWin = platform === "win32";
const isMac = platform === "darwin";

function log(msg) {
  console.log(`\x1b[36m[sricap-dev]\x1b[0m ${msg}`);
}

function error(msg) {
  console.error(`\x1b[31m[sricap-dev error]\x1b[0m ${msg}`);
}

function success(msg) {
  console.log(`\x1b[32m[sricap-dev]\x1b[0m ${msg}`);
}

function detectBinary() {
  const ext = isWin ? ".exe" : "";
  const candidates = [
    path.join(gpuiDir, "target", "debug", `sricap-rec${ext}`),
    path.join(gpuiDir, "target", "release", `sricap-rec${ext}`),
    path.join(gpuiDir, "target", "debug", `cap-gpui${ext}`),
    path.join(gpuiDir, "target", "release", `cap-gpui${ext}`),
  ];
  for (const c of candidates) {
    if (fs.existsSync(c)) return c;
  }
  return path.join(gpuiDir, "target", "debug", `sricap-rec${ext}`);
}

export function stopRunningApp() {
  log("Stopping any running sricap-rec instances...");
  if (fs.existsSync(pidFile)) {
    try {
      const pid = parseInt(fs.readFileSync(pidFile, "utf8").trim(), 10);
      if (pid && !isNaN(pid)) {
        try {
          process.kill(pid, "SIGTERM");
          log(`Sent SIGTERM to PID ${pid}`);
        } catch (_) {}
      }
    } catch (_) {}
    try { fs.unlinkSync(pidFile); } catch (_) {}
  }

  if (isWin) {
    spawnSync("taskkill", ["/F", "/IM", "sricap-rec.exe", "/T"], { stdio: "ignore" });
    spawnSync("taskkill", ["/F", "/IM", "cap-gpui.exe", "/T"], { stdio: "ignore" });
  } else {
    spawnSync("pkill", ["-9", "-f", "sricap-rec"], { stdio: "ignore" });
    spawnSync("pkill", ["-9", "-f", "cap-gpui"], { stdio: "ignore" });
  }
  success("All previous instances terminated cleanly.");
}

export function startDev({ release = false } = {}) {
  stopRunningApp();

  log(`Platform detected: ${platform} (${arch})`);
  log(`Mode: ${release ? "Release" : "Incremental Debug"}`);

  log("Building sricap-rec incrementally with cargo...");
  const cargoArgs = ["build"];
  if (release) cargoArgs.push("--release");

  const buildRes = spawnSync("cargo", cargoArgs, {
    cwd: gpuiDir,
    stdio: "inherit",
    env: {
      ...process.env,
      CARGO_INCREMENTAL: "1",
      RUST_BACKTRACE: "1",
    },
  });

  if (buildRes.status !== 0) {
    error(`Compilation failed with exit code ${buildRes.status}`);
    process.exit(1);
  }

  const binaryPath = detectBinary();
  if (!fs.existsSync(binaryPath)) {
    error(`Built executable not found at ${binaryPath}`);
    process.exit(1);
  }

  success(`Build ready: ${binaryPath}`);
  log("Launching sricap-rec locally in background...");

  fs.mkdirSync(path.dirname(pidFile), { recursive: true });

  const child = spawn(binaryPath, [], {
    detached: true,
    stdio: "inherit",
    cwd: gpuiDir,
    env: {
      ...process.env,
      RUST_LOG: process.env.RUST_LOG || "info",
    },
  });

  if (child.pid) {
    fs.writeFileSync(pidFile, child.pid.toString());
    success(`sricap-rec running locally (PID: ${child.pid})`);
  }

  child.unref();
}

// CLI entrypoint
const command = process.argv[2] || "start";
if (command === "stop") {
  stopRunningApp();
} else if (command === "start") {
  startDev({ release: process.argv.includes("--release") });
} else if (command === "restart") {
  startDev({ release: process.argv.includes("--release") });
} else {
  console.log("Usage: node scripts/dev-local.mjs [start|stop|restart] [--release]");
}
