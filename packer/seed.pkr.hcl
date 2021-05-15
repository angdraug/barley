source "nspawn" "seed" {
  clone = "base"
}

build {
  sources = ["source.nspawn.seed"]

  provisioner "apt" {
    packages = [
      # required
      "linux-image-amd64", "iproute2", "curl", "openssh-server", "gnutls-bin", "jq", "zstd",

      # optional persistent storage management
      "gdisk", "cryptsetup", "lvm2",

      # optional troubleshooting tools
      "less", "linux-perf", "sysstat", "vim-tiny",
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
    sources = ["barley-register", "zap-disk", "attach-disk"]
    destination = "/usr/local/bin/"
  }

  provisioner "file" {
    sources = ["barley-machine-key.service", "barley-register.service", "ssh-host-key.service"]
    destination = "/etc/systemd/system/"
  }

  provisioner "file" {
    source = "ssh-host-key.conf"
    destination = "/etc/ssh/sshd_config.d/key.conf"
  }

  provisioner "shell" {
    inline = [
      "sed 's/--network-veth/--network-bridge=br0/' /lib/systemd/system/systemd-nspawn@.service > /etc/systemd/system/systemd-nspawn@.service",
      "sed -i 's/^#*SystemMaxUse=.*$/SystemMaxUse=32M/' /etc/systemd/journald.conf",
      "rm /etc/ssh/ssh_host_*",
      "adduser --system --group --disabled-login --home /var/lib/barley barley",
      "chmod 755 /usr/local/bin/barley-register",
      "chmod 755 /usr/local/bin/zap-disk",
      "chmod 755 /usr/local/bin/attach-disk",
      "systemctl enable barley-machine-key barley-register ssh-host-key",
      "install -d -m 700 /root/.ssh",
    ]
  }

  post-processors {
    post-processor "shell-local" {
      environment_vars = ["MACHINE=seed"]
      script = "make-initramfs"
    }

    post-processor "shell-local" {
      command = "machinectl remove seed"
    }

    post-processor "artifice" {
      files = [
        "seed.cpio.zst",
        "seed.vmlinuz",
      ]
    }
  }
}
