all: release sower.tar.zst

release: target/release/barley

target/release/barley: Cargo.toml src/main.rs src/lib.rs
	cargo build --release

test:
	cargo test

sower.tar.zst:
	packer build -only '*.base' .
	packer build -only '*.seed' .
	packer build -only '*.sower' .
	machinectl remove base seed

envoy.tar.zst:
	packer build packer/envoy

postgres.tar.zst:
	packer build packer/postgres

synapse.tar.zst:
	packer build packer/synapse
