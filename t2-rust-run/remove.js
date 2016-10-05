#!/usr/bin/env node

var fs = require('fs-extra');
var path = require('path');
var osenv = require('osenv');

fs.remove(path.join(osenv.home(), '.tessel/rust'), function () {
  fs.remove(path.join(osenv.home(), '.tessel/sdk'), function () {
    console.log('done.');
  });
});
