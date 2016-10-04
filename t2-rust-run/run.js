#!/usr/bin/env node

var rust = require('./');
var spawn = require('child_process').spawn;
var path = require('path');

var env = Object.assign({}, process.env);
var stagingDir = "/Users/tim/.tessel/sdk/macos";
Object.assign(env, {
  STAGING_DIR: stagingDir,
  RUST_TARGET_PATH: "/Users/tim/.tessel/rust",
  PATH: path.join(stagingDir, "toolchain-mipsel_24kec+dsp_gcc-4.8-linaro_uClibc-0.9.33.2/bin") + ":" + env.PATH,
  RUSTFLAGS: "-L /Users/tim/.tessel/rust/rustlib/1.12.0",
});

var cargo = spawn('cargo', ['build', '--target=tessel2'], {
  env: env,
  stdio: ['ignore', 'inherit', 'inherit'],
});

cargo.on('error', (error) => {
  console.log(`ERROR ${error.stack}`);
});

cargo.on('close', (code) => {
  console.log(`cargo exited with code ${code}`);
});
