#!/bin/sh
git clone --depth 1 -b pass-archivepath-v3.24.0 https://github.com/angdraug/cryptpad.git cryptpad
cd cryptpad
rm -rf .git
npm install
bower install
