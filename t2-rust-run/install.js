#!/usr/bin/env node

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

var rustlibUrl = 'https://builds.tessel.io/t2/sdk/t2-rustlib-1.12.0.tar.gz';

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

function rustlibPath() {
  return fs.readdirAsync(paths.rustlib)
    .then((_) => paths.rustlib);
}

function rustTargetPath() {
  return fs.readFileAsync(paths.rustTarget)
    .then((_) => paths.rustTarget);
}

function sdkExists() {
  return toolchainPath()
  .then((path) => true)
  .catch(() => false)
}

function rustlibExists() {
  return rustlibPath()
  .then((path) => true)
  .catch(() => false)
}

function rustTargetExists() {
  return rustTargetPath()
  .then((path) => true)
  .catch(() => false)
}

function installSdk() {
  console.log('Installing SDK...');
  return sdkExists()
  .then(function (exists) {
    if (exists) {
      console.log('(SDK already installed.)')
    } else {
      return fs.mkdirpAsync(path.join(osenv.home(), '.tessel/sdk'))
        .then(() => extractSdk(download(sdkPath[myPlatform])));
    }
  });
}

function installRustlib(next) {
  console.log('Installing rustlib...');
  return rustlibExists()
  .then(function (exists) {
    if (exists) {
      console.log('(Rustlib already installed.)')
    } else {
      return fs.mkdirpAsync(path.join(osenv.home(), '.tessel/rust/rustlib'))
        .then(() => extractRustlib(download(rustlibUrl), '1.12.0'))
    }
  });
}

function installRustTarget(next) {
  console.log('Installing rust target...');
  return fs.mkdirpAsync(path.join(osenv.home(), '.tessel/rust'))
    .then(() => new Promise((resolve, reject) => {
      download('http://builds.tessel.io/t2/sdk/tessel2.json')
      .pipe(fs.createWriteStream(path.join(osenv.home(), '.tessel/rust/tessel2.json')))
      .on('finish', resolve)
      .on('error', reject)
    }))
}

function extractRustlib(sdkStream, rustVersion) {
  console.log('Downloading rustlib...');

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
      sdkStream
        .pipe(zlib.createGunzip())
        .pipe(blocks({ size: 64*1024, zeroPadding: false }))
        .pipe(extract)
        .on('finish', function () {
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

function extractSdk(sdkStream) {
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
      sdkStream
        .pipe(bz2())
        .pipe(blocks({ size: 64*1024, zeroPadding: false }))
        .pipe(extract)
        .on('finish', function () {
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

installSdk()
.then(() => installRustlib())
.then(() => installRustTarget())
.then(function () {
  console.log('SDK installed.');
})
