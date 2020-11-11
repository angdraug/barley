all:
	packer build -only '*.base' .
	packer build -only '*.seed' .
	packer build -only '*.sower' .
