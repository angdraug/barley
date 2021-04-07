#!/bin/sh
git clone --depth 1 -b 4.3.1 https://github.com/xwiki-labs/cryptpad.git cryptpad
cd cryptpad
rm -rf .git
npm install
bower install
bower cache clean
npm cache clean --force
