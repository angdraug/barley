source "nspawn" "sower" {
  clone = "base"
}

build {
  sources = ["source.nspawn.sower"]

  provisioner "apt" {
    packages = ["dnsmasq", "ipxe", "openssh-client", "gnutls-bin"]
  }

  provisioner "file" {
    source = "dnsmasq.conf"
    destination = "/etc/dnsmasq.d/barley.conf"
  }

  provisioner "file" {
    sources = ["target/release/barley", "barley-ip"]
    destination = "/usr/local/bin/"
  }

  provisioner "file" {
    sources = [
      "barley.service",
      "barley-ip.service",
      "barley-machine-key.service",
      "barley-ssh-ca.service",
    ]
    destination = "/etc/systemd/system/"
  }

  provisioner "shell" {
    inline = [
      "mkdir -p /srv/tftp /srv/barley",
      "ln -s /boot/ipxe.efi /usr/lib/ipxe/undionly.kpxe /srv/tftp/",
      "adduser --system --group --disabled-login --home /var/lib/barley barley",
      "chmod 755 /usr/local/bin/barley /usr/local/bin/barley-ip",
      "systemctl enable barley barley-ip barley-machine-key barley-ssh-ca",
    ]
  }

  provisioner "file" {
    sources = ["seed.cpio.gz", "seed.vmlinuz"]
    destination = "/srv/barley/"
  }

  post-processors {
    post-processor "shell-local" {
      inline = [
        "tar --zstd -C /var/lib/machines/sower -cf sower.tar.zst .",
        "rm -f seed.cpio.gz seed.vmlinuz",
        "machinectl remove sower",
      ]
    }

    post-processor "artifice" {
      files = ["sower.tar.zst"]
    }
  }
}
