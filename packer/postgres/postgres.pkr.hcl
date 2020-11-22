source "nspawn" "postgres" {
  variant = "minbase"
}

build {
  sources = ["source.nspawn.postgres"]

  provisioner "apt" {
    packages = ["postgresql"]
  }

  provisioner "shell" {
    inline = [
      <<EOF
        sed 's/^DHCP=yes/DHCP=yes\nIPv6AcceptRA=no/' \
          < /lib/systemd/network/80-container-host0.network \
          > /etc/systemd/network/80-container-host0.network
      EOF
    ]
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
