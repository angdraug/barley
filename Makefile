all: release sower.tar.zst

release: target/release/barley

target/release/barley: Cargo.toml src/main.rs src/lib.rs
	cargo build --release

test:
	cargo test

sower.tar.zst:
	packer build -only '*.base' packer/barley
	packer build -only '*.seed' packer/barley
	packer build -only '*.sower' packer/barley
	machinectl remove base seed

envoy.tar.zst:
	packer build packer/envoy
	machinectl remove envoy

postgres.tar.zst:
	packer build packer/postgres
	machinectl remove postgres

synapse.tar.zst:
	packer build packer/synapse
	machinectl remove synapse
