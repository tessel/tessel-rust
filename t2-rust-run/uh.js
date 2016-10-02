var spawn = require('child_process').spawn;

function rustVersion() {
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
        reject(new Error('Could not identify rust version.'));
      } else {
        resolve(version);
      }
    });
  });
}

rustVersion()
.then(function (v) {
  console.log(v);
});
