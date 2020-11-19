all: release
	packer build -only '*.base' .
	packer build -only '*.seed' .
	packer build -only '*.sower' .
	machinectl remove base seed

release: target/release/barley

target/release/barley: Cargo.toml src/main.rs src/lib.rs
	cargo build --release

test:
	cargo test
