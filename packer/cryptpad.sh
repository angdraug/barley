#!/bin/sh
git clone --depth 1 -b 3.24.0 https://github.com/xwiki-labs/cryptpad.git cryptpad
cd cryptpad
rm -rf .git
npm install
bower install
