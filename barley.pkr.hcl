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
    packages = ["dnsmasq", "nginx-light", "pxelinux", "syslinux-common"]
  }

  provisioner "file" {
    sources = ["seed.cpio.gz", "seed.vmlinuz"]
    destination = "/var/www/html/"
  }

  provisioner "shell" {
    inline = [
      "mkdir -p /srv/tftp/pxelinux.cfg",
      "ln -s /usr/lib/PXELINUX/lpxelinux.0 /srv/tftp/",
      "ln -s /usr/lib/syslinux/modules/bios/ldlinux.c32 /srv/tftp/",
    ]
  }

  provisioner "file" {
    source = "pxelinux.cfg"
    destination = "/srv/tftp/pxelinux.cfg/default"
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
}
