var fs = require('fs-extra-promise');
var osenv = require('osenv');
var path = require('path');
var tar = require('tar-fs')
var request = require('request');
var bz2 = require('unbzip2-stream');
var blocks = require('block-stream2');
var tmp = require('tmp');
var zlib = require('zlib');
var Promise = require('bluebird');
var Progress = require('progress');
var createHash = require('sha.js')
var Transform = require('stream').Transform;
var spawn = require('child_process').spawn;

var rustVersion = exports.rustVersion = () => {
  return new Promise((resolve, reject) => {
    var rustc = spawn('rustc', ['-V'])
    var stdout = [];
    rustc.stdout.on('data', (data) => {
      stdout.push(data);
    })
    rustc.stdout.on('close', () => {
      var out = Buffer.concat(stdout).toString();
      var version = out.match(/^rustc\s+(\S+)/)[1];

      if (!version) {
        reject(new Error('Could not identify locally installed rust version.'));
      } else {
        resolve(version);
      }
    });
  });
}

function sha256stream() {
  var sha256 = createHash('sha256');
  var stream = new Transform();
  stream._transform = function (chunk, encoding, callback) {
    this.push(chunk);
    sha256.update(chunk);
    callback();
  };
  stream.on('finish', function () {
    stream.emit('sha256', sha256.digest('hex'));
  });
  return stream;
}

function tmpdir() {
  return new Promise((resolve, reject) => {
    tmp.dir(function (err, tmppath, cleanup) {
      if (err) {
        reject(err);
      } else {
        resolve({
          path: tmppath,
          cleanup: cleanup,
        });
      }
    });
  });
}

var myPlatform = 'macos';

var paths = {
  sdk: path.join(osenv.home(), '.tessel/sdk'),
  rustlib: path.join(osenv.home(), '.tessel/rust/rustlib'),
  rustTarget: path.join(osenv.home(), '.tessel/rust/tessel2.json'),
};

var sdkPath = {
  'macos': 'https://builds.tessel.io/t2/sdk/t2-sdk-macos-x86_64.tar.bz2',
  'linux': 'https://builds.tessel.io/t2/sdk/t2-sdk-linux-x86_64.tar.bz2',
};

var rustlibUrl = 'https://builds.tessel.io/t2/sdk/t2-rustlib-VERSION.tar.gz';

function toolchainPath() {
  return fs.readdirAsync(path.join(paths.sdk, myPlatform))
    .then((values) => new Promise((resolve, reject) => {
      for (var i = 0; i < values.length; i++) {
        if (values[i].match(/^toolchain\-/)) {
          return resolve(path.join(paths.sdk, myPlatform, values[i]));
        }
      }
      return reject(new Error("No toolchain found."));
    }));
}

// Checks is CHECKSUM file in our SDK equals our expected checksum.
// This will resolve with checking that the SDK exists and matches the checksum.
function checkSdk(checksumVerify) {
  return fs.readFileAsync(path.join(paths.sdk, myPlatform, "CHECKSUM"))
  .then((checksum) => ({
    exists: true,
    valid: checksumVerify == checksum,
  }), (_) => ({
    exists: false,
    valid: false
  }))
}

function checkRustlib(rustv, checksumVerify) {
  return fs.readFileAsync(path.join(paths.rustlib, rustv, "CHECKSUM"))
  .then((checksum) => ({
    exists: true,
    valid: checksumVerify == checksum,
  }), (_) => ({
    exists: false,
    valid: false
  }))
}

function checkRustTarget(checksumVerify) {
  return new Promise((resolve, reject) => {
    try {
      fs.createReadStream(paths.rustTarget)
      .pipe(sha256stream())
      .on('sha256', (checksum) => {
        checksum = checksum + '  ' + path.basename(paths.rustTarget) + '\n';
        resolve({
          exists: true,
          valid: checksumVerify == checksum,
        })
      });
    } catch (e) {
      resolve({
        exists: false,
        valid: false,
      })
    }
  });
}

var installSdk = exports.installSdk = () => {
  var url = sdkPath[myPlatform];
  var checksumVerify = null;

  return downloadString(url + '.sha256')
  .then((checksum) => {
    checksumVerify = checksum;
    return checkSdk(checksumVerify);
  })
  .then((check) => {
    if (check.exists && check.valid) {
      console.error('Latest SDK already installed.');
      return;
    } else if (!check.exists) {
      console.error('Installing SDK...')
    } else {
      console.error('Updating SDK...')
    }

    return fs.mkdirpAsync(path.join(osenv.home(), '.tessel/sdk'))
      .then(() => extractSdk(checksumVerify, path.basename(url), download(url)));
  });
}

var installRustlib = exports.installRustlib = (next) => {
  var url = null;
  var checksumVerify = null;
  var rustv = null;
  var pkgname = null;

  return rustVersion()
  .then((_rustv) => {
    rustv = _rustv;
    pkgname = 'MIPS libstd v' + rustv
    url = rustlibUrl.replace('VERSION', rustv);

    return downloadString(url + '.sha256')
  })
  .catch((err) => {
    throw new Error('Could not find a MIPS libstd matching rust version ' + rustv + '. Only stable Rust versions are supported.');
  })
  .then((checksum) => {
    checksumVerify = checksum;
    return checkRustlib(rustv, checksumVerify);
  })
  .then((check) => {
    if (check.exists && check.valid) {
      console.error(`Latest ${pkgname} already installed.`);
      return;
    } else if (!check.exists) {
      console.error(`Installing ${pkgname}...`)
    } else {
      console.error(`Updating ${pkgname}...`)
    }

    return fs.mkdirpAsync(path.join(osenv.home(), '.tessel/rust/rustlib'))
      .then(() => extractRustlib(checksumVerify, path.basename(url), download(url), rustv))
  });
}

var installRustTarget = exports.installRustTarget = (next) => {
  var url = 'http://builds.tessel.io/t2/sdk/tessel2.json';
  var checksumVerify = null;

  return downloadString(url + '.sha256')
  .then((checksum) => {
    checksumVerify = checksum;
    return checkRustTarget(checksumVerify);
  })
  .then((check) => {
    if (check.exists && check.valid) {
      console.error('Latest target.json already installed.');
      return;
    } else if (!check.exists) {
      console.error('Installing target.json...')
    } else {
      console.error('Updating target.json...')
    }

    return fs.mkdirpAsync(path.join(osenv.home(), '.tessel/rust'))
      .then(() => downloadString(url))
      .then((target) => {
        var sha256 = createHash('sha256');
        var checksum = sha256.update(target).digest('hex') + '  ' + path.basename(url) + '\n';

        // Check sum.
        if (checksum != checksumVerify) {
          throw new Error('Checksum for downloaded target.json does not match!');
        }

        fs.writeFileSync(path.join(osenv.home(), '.tessel/rust/tessel2.json'), target);
      });
  });
}

function extractSdk(checksumVerify, filename, sdkStream) {
  console.log('Downloading SDK...');

  var root = path.join(osenv.home(), '.tessel/sdk', 'macos');

  return tmpdir()
  .then((destdir) => {
    // Exract tarball to destination.
    var extract = tar.extract(destdir.path, {
      strip: 2,
      ignore: function(name) {
        // Ignore self-directory.
        return path.normalize(name + '/') == path.normalize(destdir.path + '/');
      }
    });

    function tryCleanup() {
      try {
        destdir.cleanup();
      } catch (e) { }
    }

    return new Promise((resolve, reject) => {
      var checksum = '';
      sdkStream
        .pipe(sha256stream())
        .on('sha256', function (sha256) {
          checksum = sha256 + '  ' + filename + '\n';
        })
        .pipe(bz2())
        .pipe(blocks({ size: 64*1024, zeroPadding: false }))
        .pipe(extract)
        .on('finish', function () {
          // Check sum.
          if (checksum != checksumVerify) {
            return reject(new Error('Checksum for downloaded SDK does not match!'));
          }

          // Write out CHECKSUM file.
          fs.writeFileSync(path.join(destdir.path, "CHECKSUM"), checksum);

          // Remove the old SDK directory.
          fs.removeAsync(root)
          .then(() => {
            // Move temporary directory to target destination.
            return fs.moveAsync(destdir.path, root);
          })
          .finally(() => {
            tryCleanup();
          })
          .then(resolve)
          .catch(reject);
        })
        .on('error', function (err) {
          tryCleanup();
          reject(err);
        })
      });
  });
}

function extractRustlib(checksumVerify, filename, sdkStream, rustVersion) {
  console.log('Downloading MIPS libstd...');

  var root = path.join(osenv.home(), '.tessel/rust/rustlib', rustVersion);

  return tmpdir()
  .then((destdir) => {
    // Exract tarball to destination.
    var extract = tar.extract(destdir.path, {
      strip: 0,
      ignore: function(name) {
        // Ignore self-directory.
        return path.normalize(name + '/') == path.normalize(destdir.path + '/');
      }
    });

    function tryCleanup() {
      try {
        destdir.cleanup();
      } catch (e) { }
    }

    return new Promise((resolve, reject) => {
      var checksum = '';
      sdkStream
        .pipe(sha256stream())
        .on('sha256', function (sha256) {
          checksum = sha256 + '  ' + filename + '\n';
        })
        .pipe(zlib.createGunzip())
        .pipe(blocks({ size: 64*1024, zeroPadding: false }))
        .pipe(extract)
        .on('finish', function () {
          // Check sum.
          if (checksum != checksumVerify) {
            return reject(new Error('Checksum for downloaded MIPS libstd does not match!'));
          }

          // Write out CHECKSUM file.
          fs.writeFileSync(path.join(destdir.path, "CHECKSUM"), checksum);

          // Remove the old SDK directory.
          fs.removeAsync(root)
          .then(() => {
            // Move temporary directory to target destination.
            fs.moveAsync(destdir.path, root)
          })
          .finally(() => {
            tryCleanup();
          })
          .then(resolve)
          .catch(reject);
        })
        .on('error', function (err) {
          tryCleanup();
          reject(err);
        })
      });
  });
}

function downloadString(url) {
  return new Promise((resolve, reject) => {
    request(url, (error, response, body) => {
      if (!error && response.statusCode == 200) {
        resolve(body);
      } else {
        reject(error || response.statusCode);
      }
    })
  });
}

function download(url) {
  var req = request.get(url);

  // When we receive the response
  req.on('response', (res) => {

    // Parse out the length of the incoming bundle
    var contentLength = parseInt(res.headers['content-length'], 10);

    // Create a new progress bar
    var bar = new Progress('     [:bar] :percent :etas remaining', {
      clear: true,
      complete: '=',
      incomplete: ' ',
      width: 20,
      total: contentLength
    });

    // When we get incoming data, update the progress bar
    res.on('data', (chunk) => {
      bar.tick(chunk.length);
    });
  });

  return req;
}
