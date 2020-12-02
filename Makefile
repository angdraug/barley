all: release sower.tar.zst

release: target/release/barley

target/release/barley: Cargo.toml src/main.rs src/lib.rs
	cargo build --release

test:
	cargo test

base.tar.zst:
	packer build packer/base.pkr.hcl

sower.tar.zst: base.tar.zst
	packer build packer/seed.pkr.hcl
	packer build packer/sower.pkr.hcl
	machinectl remove seed

images: envoy.tar.zst nginx.tar.zst postgres.tar.zst synapse.tar.zst

cryptpad.tar.zst: base.tar.zst
	packer build packer/cryptpad.pkr.hcl
	machinectl remove cryptpad

envoy.tar.zst: base.tar.zst
	packer build packer/envoy.pkr.hcl
	machinectl remove envoy

nginx.tar.zst: base.tar.zst
	packer build packer/nginx.pkr.hcl
	machinectl remove nginx

postgres.tar.zst: base.tar.zst
	packer build packer/postgres.pkr.hcl
	machinectl remove postgres

synapse.tar.zst: base.tar.zst
	packer build packer/synapse.pkr.hcl
	machinectl remove synapse
