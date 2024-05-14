#!/usr/bin/env node

import { spawnSync } from "child_process";

const isMusl = () => {
  let musl = false
  if (process.platform === 'linux') {
    musl = isMuslFromFilesystem()
    if (musl === null) {
      musl = isMuslFromReport()
    }
    if (musl === null) {
      musl = isMuslFromChildProcess()
    }
  }
  return musl
}

const isFileMusl = (f) => f.includes('libc.musl-') || f.includes('ld-musl-')

const isMuslFromFilesystem = () => {
  try {
    return readFileSync('/usr/bin/ldd', 'utf-8').includes('musl')
  } catch {
    return null
  }
}

const isMuslFromReport = () => {
  const report = typeof process.report.getReport === 'function' ? process.report.getReport() : null
  if (!report) {
    return null
  }
  if (report.header && report.header.glibcVersionRuntime) {
    return false
  }
  if (Array.isArray(report.sharedObjects)) {
    if (report.sharedObjects.some(isFileMusl)) {
      return true
    }
  }
  return false
}

const isMuslFromChildProcess = () => {
  try {
    return require('child_process').execSync('ldd --version', { encoding: 'utf8' }).includes('musl')
  } catch (e) {
    // If we reach this case, we don't know if the system is musl or not, so is better to just fallback to false
    return false
  }
}

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
    case isMusl():
      packageSuffix = '-musl';
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
