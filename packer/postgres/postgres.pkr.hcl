source "nspawn" "postgres" {
  variant = "minbase"
}

build {
  sources = ["source.nspawn.postgres"]

  provisioner "apt" {
    packages = ["postgresql"]
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
