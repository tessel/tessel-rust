#!/usr/bin/env node

var rust = require('./');
var spawn = require('child_process').spawn;
var path = require('path');

var rustv;
var toolchainPath;
var stagingDir;
var rustTargetPath;
var rustlibPath;

rust.rustVersion()
.then((_rustv) => {
  rustv = _rustv;
  return rust.checkSdk()
})
.then((check) => {
  if (!check.exists) {
    throw new Error('SDK not installed.');
  }

  stagingDir = check.path;
  return rust.checkRustlib(rustv)
})
.then((check) => {
  if (!check.exists) {
    throw new Error(`MIPS libstd v${rustv} not installed.`);
  }

  rustlibPath = check.path;
  return rust.checkRustTarget()
})
.then((check) => {
  if (!check.exists) {
    throw new Error('target.json not installed.');
  }

  rustTargetPath = path.dirname(check.path);
  return rust.toolchainPath()
})
.then((_toolchainPath) => {
  toolchainPath = _toolchainPath;

  var env = Object.assign({}, process.env);
  Object.assign(env, {
    STAGING_DIR: stagingDir,
    RUST_TARGET_PATH: rustTargetPath,
    PATH: path.join(toolchainPath, "bin") + ":" + env.PATH,
    RUSTFLAGS: "-L " + rustlibPath,
  });

  var cargo = spawn('cargo', ['build', '--target=tessel2'], {
    env: env,
    stdio: ['ignore', 'inherit', 'inherit'],
  });

  cargo.on('error', (error) => {
    console.error(`ERROR ${error.stack}`);
  });

  cargo.on('close', (code) => {
    console.error(`cargo exited with code ${code}`);
    process.on('exit', () => {
      process.exit(code);
    });
  });
}, (e) => {
  console.error('Could not find all the components for cross-compiling Rust.')
  console.error(e.message);
  console.error('Please run "t2 sdk install" and try again.')
  process.on('exit', () => {
    process.exit(1);
  });
});
