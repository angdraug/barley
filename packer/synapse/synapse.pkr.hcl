source "nspawn" "synapse" {
  variant = "minbase"
}

build {
  sources = ["source.nspawn.synapse"]

  provisioner "apt" {
    packages = ["matrix-synapse", "python3-psycopg2"]
  }

  post-processors {
    post-processor "shell-local" {
      inline = [
        "tar --zstd -C /var/lib/machines/synapse -cf synapse.tar.zst .",
      ]
    }

    post-processor "artifice" {
      files = ["synapse.tar.zst"]
    }
  }
}
