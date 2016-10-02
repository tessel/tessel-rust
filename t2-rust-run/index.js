#!/usr/bin/env node

console.log('hi');

var fs = require('fs-extra');
var osenv = require('osenv');
var path = require('path');
var tar = require('tar-fs')
var request = require('request');
var bz2 = require('unbzip2-stream');
var blocks = require('block-stream2');
var tmp = require('tmp');

fs.mkdirp(path.join(osenv.home(), '.tessel/sdk'), (err) => {
  // downloadTgz('https://s3.amazonaws.com/builds.tessel.io/t2/OpenWRT+SDK/OpenWrt-SDK-ramips-mt7620_gcc-4.8-linaro_uClibc-0.9.33.2.Darwin-x86_64.tar.bz2')
  extractSdk(fs.createReadStream('./t2-0.0.16-sdk-macos.tar.bz2'))
  .then(function () {
    console.log('done')
  })
  .catch(function (err) {
    console.log(err);
  })
});



function extractSdk(sdkStream) {
  return new Promise((resolve, reject) => {
    console.log('Downloading files...');

    tmp.dir(function (err, tmppath, cleanup) {
      // Fetch the list of available files
      var root = path.join(osenv.home(), '.tessel/sdk', 'macos');
      var extract = tar.extract(tmppath, {
        strip: 2,
        ignore: function(name) {
          // console.log(path.normalize(name + '/'), path.normalize(root + '/'));
          return path.normalize(name + '/') == path.normalize(tmppath + '/');
          // return path.extname(name) === '.bin' // ignore .bin files when packing
        }
      });

      // extract.on('error', function (err) {
      //   console.log('skipping', err.stack);
      // });
      //
      // extract.once('finish', () => {
      //   for (var key in files) {
      //     var file = files[key];
      //     if (!file.length) {
      //       return reject(new Error('Fetched file was not formatted properly.'));
      //     }
      //   }
      //   log.info('Download complete!');
      //   return resolve(files);
      // });

      console.log('extracting to:', tmppath);

      sdkStream
        .pipe(bz2())
        .pipe(blocks({ size: 64*1024, zeroPadding: false }))
        .pipe(extract)
        .on('finish', function () {
          // Remove the old SDK directory.
          fs.remove(root, function (err) {
            if (err) {
              reject(err);
            } else {
              // Move temporary directory to target destination.
              fs.move(tmppath, root, function (err) {
                try {
                  cleanup();
                } catch (e) { }
                console.log('moved to:', root);
                if (err) {
                  reject(err);
                } else {
                  resolve();
                }
              });
            }
          });
        })
        .on('error', function (err) {
          try {
            cleanup();
          } catch (e) { }
          reject(err);
        })
    });

    // var req = request.get(tgzUrl);
    //
    // // When we receive the response
    // req.on('response', (res) => {
    //
    //   // Parse out the length of the incoming bundle
    //   var contentLength = parseInt(res.headers['content-length'], 10);
    //
    //   // Create a new progress bar
    //   // var bar = new Progress('     [:bar] :percent :etas remaining', {
    //   //   clear: true,
    //   //   complete: '=',
    //   //   incomplete: ' ',
    //   //   width: 20,
    //   //   total: contentLength
    //   // });
    //
    //   // When we get incoming data, update the progress bar
    //   // res.on('data', (chunk) => {
    //   //   bar.tick(chunk.length);
    //   // });
    //
    //   // unzip and extract the binary tarball
    //   res.pipe(bz2()).pipe(extract).on('end', function () {
    //     resolve();
    //   })
    // });
  });
};
