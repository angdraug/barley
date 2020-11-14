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
      "linux-image-amd64", "openssh-server",

      # troubleshooting tools (optional)
      "iproute2", "less", "linux-perf", "sysstat", "vim-tiny",
    ]
  }

  provisioner "file" {
    source = "ssh-host-key.service"
    destination = "/etc/systemd/system/ssh-host-key.service"
  }

  provisioner "shell" {
    inline = [
      "rm /etc/ssh/ssh_host_*",
      "echo HostKey=/etc/ssh/ssh_host_ed25519_key > /etc/ssh/sshd_config.d/host-key",
      "/bin/systemctl enable ssh-host-key.service",
    ]
  }

  provisioner "shell" {
    inline = ["/usr/bin/install -d -m 700 /root/.ssh"]
  }

  provisioner "file" {
    source = "id_ed25519.pub"
    destination = "/root/.ssh/authorized_keys"
  }

  provisioner "file" {
    sources = [
      "network/20-br0.netdev",
      "network/30-br0-bind.network",
      "network/40-br0.network",
    ]
    destination = "/etc/systemd/network/"
  }

  provisioner "shell" {
    inline = ["/bin/sed -i 's/^#*SystemMaxUse=.*$/SystemMaxUse=32M/' /etc/systemd/journald.conf"]
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
    packages = ["dnsmasq", "nginx-light", "ipxe"]
  }

  provisioner "file" {
    sources = ["seed.cpio.gz", "seed.vmlinuz", "seed.ipxe"]
    destination = "/var/www/html/"
  }

  provisioner "shell" {
    inline = [
      "mkdir -p /srv/tftp",
      "ln -s /boot/ipxe.efi /srv/tftp/",
      "ln -s /usr/lib/ipxe/undionly.kpxe /srv/tftp/",
    ]
  }

  provisioner "file" {
    source = "dnsmasq.conf"
    destination = "/etc/dnsmasq.d/sower.conf"
  }

  provisioner "file" {
    source = "update-sower-ipaddress"
    destination = "/usr/local/bin/update-sower-ipaddress"
  }

  provisioner "file" {
    source = "update-sower-ipaddress.service"
    destination = "/etc/systemd/system/update-sower-ipaddress.service"
  }

  provisioner "shell" {
    inline = [
      "/bin/chmod 755 /usr/local/bin/update-sower-ipaddress",
      "/bin/systemctl enable update-sower-ipaddress.service",
    ]
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
