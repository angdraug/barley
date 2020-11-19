source "nspawn" "base" {
    variant = "minbase"
}

build {
  sources = ["source.nspawn.base"]
}

source "nspawn" "seed" {
  clone = "base"
}

build {
  sources = ["source.nspawn.seed"]

  provisioner "apt" {
    packages = [
      # required
      "linux-image-amd64", "openssh-server", "curl",

      # troubleshooting tools (optional)
      "iproute2", "less", "linux-perf", "sysstat", "vim-tiny",
    ]
  }

  provisioner "file" {
    sources = [
      "network/20-br0.netdev",
      "network/30-br0-bind.network",
      "network/40-br0.network",
    ]
    destination = "/etc/systemd/network/"
  }

  provisioner "file" {
    source = "register-barley-seed"
    destination = "/usr/local/bin/register-barley-seed"
  }

  provisioner "file" {
    sources = ["register-barley-seed.service", "ssh-host-key.service"]
    destination = "/etc/systemd/system/"
  }

  provisioner "shell" {
    inline = [
      "/bin/sed -i 's/^#*SystemMaxUse=.*$/SystemMaxUse=32M/' /etc/systemd/journald.conf",
      "rm /etc/ssh/ssh_host_*",
      "echo HostKey=/etc/ssh/ssh_host_ed25519_key > /etc/ssh/sshd_config.d/host-key",
      "/bin/chmod 755 /usr/local/bin/register-barley-seed",
      "/bin/systemctl enable register-barley-seed ssh-host-key.service",
      "/usr/bin/install -d -m 700 /root/.ssh",
    ]
  }

  provisioner "file" {
    source = "id_ed25519.pub"
    destination = "/root/.ssh/authorized_keys"
  }

  post-processors {
    post-processor "shell-local" {
      environment_vars = ["MACHINE=seed"]
      script = "make-initramfs"
    }

    post-processor "artifice" {
      files = [
        "seed.cpio.gz",
        "seed.vmlinuz",
      ]
    }
  }
}

source "nspawn" "sower" {
  clone = "base"
}

build {
  sources = ["source.nspawn.sower"]

  provisioner "apt" {
    packages = ["dnsmasq", "ipxe"]
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
    sources = ["barley.service", "update-barley-ipaddress.service"]
    destination = "/etc/systemd/system/"
  }

  provisioner "shell" {
    inline = [
      "mkdir -p /srv/tftp /srv/barley",
      "ln -s /boot/ipxe.efi /usr/lib/ipxe/undionly.kpxe /srv/tftp/",
      "install -d -m 775 -g www-data /var/lib/barley",
      "/bin/chmod 755 /usr/local/bin/barley /usr/local/bin/update-barley-ipaddress",
      "/bin/systemctl enable barley.service update-barley-ipaddress.service",
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
