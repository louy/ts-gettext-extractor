#!/usr/bin/env node

import { spawnSync } from "child_process";

/**
 * Returns the executable path which is located inside `node_modules`
 * The naming convention is ${package}-${os}-${arch}
 * If the platform is `win32` or `cygwin`, executable will include a `.exe` extension.
 * @see https://nodejs.org/api/os.html#osarch
 * @see https://nodejs.org/api/os.html#osplatform
 * @example "x/xx/node_modules/ts-gettext-extractor-darwin-arm64"
 */
function getExePath() {
  const arch = process.arch;
  let os = process.platform;
  let extension = "";
  let packageSuffix = ''

  switch (true) {
    case ["win32", "cygwin"].includes(process.platform):
      os = "windows";
      extension = ".exe";
      break;
    case os === 'linux':
      packageSuffix = '-gnu';
      break;
  }

  try {
    // Since the binary will be located inside `node_modules`, we can simply call `require.resolve`
    return require.resolve(`ts-gettext-extractor-${os}-${arch}${packageSuffix}/bin/ts-gettext-extractor${extension}`);
  } catch (e) {
    throw new Error(
      `Couldn't find binary inside node_modules for ${os}-${arch}`
    );
  }
}

/**
 * Runs the binary with args using nodejs spawn
 */
function run() {
  const args = process.argv.slice(2);
  const processResult = spawnSync(getExePath(), args, { stdio: "inherit" });
  process.exit(processResult.status ?? 0);
}

run();
