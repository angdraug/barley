all: release sower.tar.zst

release: target/release/barley target/release/sow

target/release/barley: Cargo.toml src/lib.rs src/ssh.rs src/tls.rs src/main.rs
	cargo build --release --bin barley
	strip target/release/barley

target/release/sow: Cargo.toml src/lib.rs src/ssh.rs src/tls.rs src/bin/sow.rs
	cargo build --release --bin sow
	strip target/release/sow

test:
	cargo test

install:
	install -m 755 target/release/sow /usr/local/bin

base.tar.zst:
	packer build packer/base.pkr.hcl

sower.tar.zst: base.tar.zst
	packer build packer/seed.pkr.hcl
	packer build packer/sower.pkr.hcl

images: cryptpad.tar.zst envoy.tar.zst nginx.tar.zst postgres.tar.zst synapse.tar.zst

cryptpad.tar.zst: base.tar.zst
	packer build packer/cryptpad.pkr.hcl

envoy.tar.zst: base.tar.zst
	packer build packer/envoy.pkr.hcl

nginx.tar.zst: base.tar.zst
	packer build packer/nginx.pkr.hcl

postgres.tar.zst: base.tar.zst
	packer build packer/postgres.pkr.hcl

synapse.tar.zst: base.tar.zst
	packer build packer/synapse.pkr.hcl

clean:
	rm -f *.tar.zst
	rm -f target/release/barley target/release/sow
