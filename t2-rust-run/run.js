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

  return new Promise((resolve, reject) => {
    var cargo = spawn('cargo', ['metadata', '--no-deps'], {
      stdio: ['ignore', 'pipe', 'ignore'],
    });

    var buffers = [];
    cargo.stdout.on('data', data => buffers.push(data));
    cargo.stdout.on('finish', () => {
      var metadata = JSON.parse(Buffer.concat(buffers).toString());

      // Get first package.
      var pkg = metadata.packages.pop();
      var bins = pkg.targets.filter(target => target.kind.indexOf('bin') > -1);

      // Filter by --bin argument.
      var idx = process.argv.indexOf('--bin');
      var name = idx > -1 ? process.argv[idx + 1] : null;
      var validbins = bins;
      if (name != null) {
        validbins = validbins.filter(bin => bin.name == process.argv[idx + 1]);
      }

      // Throw if multiple bins exist.
      if (validbins.length == 0) {
        if (name) {
          console.error(`No binary target "${name}" exists for this Rust crate.\nMake sure you specify a valid binary using --bin. e.g.:`)
          bins.forEach(bin => {
            console.error('    t2 run Cargo.toml --bin', bin.name);
          })
        } else {
          console.error('No binary targets exist for this Rust crate.\nPlease add a binary and try again.')
        }
        process.exit(1);
      }
      if (validbins.length > 1) {
        console.error('Multiple binary targets exist for this Rust crate.\nPlease specify one by name with --bin. e.g.:')
        bins.forEach(bin => {
          console.error('    t2 run Cargo.toml --bin', bin.name);
        })
        process.exit(1);
      }

      var out = validbins[0];
      var dest = path.join(path.dirname(pkg.manifest_path), 'target/tessel2/release', out.name);
      resolve({
        name: out.name,
        path: dest
      });
    })
  });
})
.then((dest) => {
  var env = Object.assign({}, process.env);
  Object.assign(env, {
    STAGING_DIR: stagingDir,
    RUST_TARGET_PATH: rustTargetPath,
    PATH: path.join(toolchainPath, "bin") + ":" + env.PATH,
    RUSTFLAGS: "-L " + rustlibPath,
  });

  var cargo = spawn('cargo', ['build', '--target=tessel2', '--bin', dest.name, '--release'], {
    env: env,
    stdio: ['ignore', 'inherit', 'inherit'],
  });

  cargo.on('error', (error) => {
    console.error(`ERROR ${error.stack}`);
  });

  cargo.on('close', (code) => {
    if (code != 0) {
      process.on('exit', () => {
        process.exit(code);
      });
    }

    console.log('TODO: copy this file to tessel and run it:');
    console.log(dest.path);
  });
}, (e) => {
  console.error('Could not find all the components for cross-compiling Rust.')
  console.error(e.message);
  console.error('Please run "t2 sdk install" and try again.')
  process.on('exit', () => {
    process.exit(1);
  });
});
