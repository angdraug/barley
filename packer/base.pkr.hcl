source "nspawn" "base" {
    variant = "minbase"
}

build {
  sources = ["source.nspawn.base"]

  post-processors {
    post-processor "shell-local" {
      inline = [
        "tar --zstd -C /var/lib/machines/base -cf base.tar.zst .",
      ]
    }

    post-processor "artifice" {
      files = ["base.tar.zst"]
    }
  }
}
