#!/bin/sh -eux
export PATH="${PATH}:/opt/ruby/bin:/opt/mastodon/bin"

git clone --depth 1 -b v4.0.2 https://github.com/mastodon/mastodon.git .
rm -rf .git

bundle config set --local deployment 'true'
bundle config set --local without 'development test'
bundle config set silence_root_warning true
bundle install -j"$(nproc)"

yarnpkg install --pure-lockfile --network-timeout 600000

export RAILS_ENV=production
export NODE_ENV=production
export NODE_OPTIONS=--openssl-legacy-provider
export RAILS_SERVE_STATIC_FILES=true
export BIND=0.0.0.0
OTP_SECRET=precompile_placeholder SECRET_KEY_BASE=precompile_placeholder rails assets:precompile

yarnpkg cache clean
