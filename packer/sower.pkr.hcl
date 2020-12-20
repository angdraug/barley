source "nspawn" "sower" {
  clone = "base"
}

build {
  sources = ["source.nspawn.sower"]

  provisioner "apt" {
    packages = ["dnsmasq", "ipxe", "openssh-client"]
  }

  provisioner "file" {
    source = "dnsmasq.conf"
    destination = "/etc/dnsmasq.d/barley.conf"
  }

  provisioner "file" {
    sources = ["target/release/barley", "update-barley-ipaddress"]
    destination = "/usr/local/bin/"
  }

  provisioner "file" {
    sources = ["barley.service", "barley-ca.service", "update-barley-ipaddress.service"]
    destination = "/etc/systemd/system/"
  }

  provisioner "shell" {
    inline = [
      "mkdir -p /srv/tftp /srv/barley",
      "ln -s /boot/ipxe.efi /usr/lib/ipxe/undionly.kpxe /srv/tftp/",
      "install -d -m 775 -g www-data /var/lib/barley",
      "/bin/chmod 755 /usr/local/bin/barley /usr/local/bin/update-barley-ipaddress",
      "/bin/systemctl enable barley.service barley-ca.service update-barley-ipaddress.service",
    ]
  }

  provisioner "file" {
    sources = ["seed.cpio.gz", "seed.vmlinuz"]
    destination = "/srv/barley/"
  }

  post-processors {
    post-processor "shell-local" {
      inline = [
        "rm -f seed.cpio.gz seed.vmlinuz",
        "tar --zstd -C /var/lib/machines/sower -cf sower.tar.zst .",
      ]
    }

    post-processor "artifice" {
      files = ["sower.tar.zst"]
    }
  }
}
