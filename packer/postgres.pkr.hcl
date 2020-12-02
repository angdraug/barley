source "nspawn" "postgres" {
  clone = "base"
}

build {
  sources = ["source.nspawn.postgres"]

  provisioner "apt" {
    packages = ["postgresql"]
  }

  provisioner "shell" {
    script = "no-ipv6.sh"
  }

  post-processors {
    post-processor "shell-local" {
      inline = [
        "tar --zstd -C /var/lib/machines/postgres -cf postgres.tar.zst .",
      ]
    }

    post-processor "artifice" {
      files = ["postgres.tar.zst"]
    }
  }
}
