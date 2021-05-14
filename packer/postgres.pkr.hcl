source "nspawn" "postgres" {
  clone = "base"
}

build {
  sources = ["source.nspawn.postgres"]

  provisioner "shell" {
    inline = [
      "mkdir -p /etc/postgresql-common/createcluster.d",
      "echo create_main_cluster = false > /etc/postgresql-common/createcluster.d/create.conf",
    ]
  }

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
        "machinectl remove postgres",
      ]
    }

    post-processor "artifice" {
      files = ["postgres.tar.zst"]
    }
  }
}
