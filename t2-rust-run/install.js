#!/usr/bin/env node

var rust = require('./');

rust.installSdk()
.then(() => rust.installRustlib())
.then(() => rust.installRustTarget())
.then(() => {
  console.log('SDK installed.');
})
