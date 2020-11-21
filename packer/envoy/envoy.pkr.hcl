source "nspawn" "envoy" {
  variant = "minbase"
}

build {
  sources = ["source.nspawn.envoy"]

  provisioner "shell-local" {
    command = "curl -sL https://getenvoy.io/gpg | gpg --dearmor -o getenvoy.gpg"
  }

  provisioner "apt" {
    packages = ["getenvoy-envoy"]
    sources = ["deb [arch=amd64] https://dl.bintray.com/tetrate/getenvoy-deb bullseye stable"]
    keys = ["getenvoy.gpg"]
  }

  post-processors {
    post-processor "shell-local" {
      inline = [
        "rm -f seed.cpio.gz seed.vmlinuz",
        "tar --zstd -C /var/lib/machines/envoy -cf envoy.tar.zst .",
      ]
    }

    post-processor "artifice" {
      files = ["envoy.tar.zst"]
    }
  }
}
