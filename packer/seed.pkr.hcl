source "nspawn" "seed" {
  clone = "base"
}

build {
  sources = ["source.nspawn.seed"]

  provisioner "apt" {
    packages = [
      # required
      "linux-image-amd64", "iproute2", "curl", "openssh-server",

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
    sources = ["register-barley-seed", "zap-disk", "attach-disk"]
    destination = "/usr/local/bin/"
  }

  provisioner "file" {
    sources = ["register-barley-seed.service", "ssh-host-key.service"]
    destination = "/etc/systemd/system/"
  }

  provisioner "file" {
    source = "ssh-host-key.conf"
    destination = "/etc/ssh/sshd_config.d/key.conf"
  }

  provisioner "shell" {
    inline = [
      "/bin/sed 's/--network-veth/--network-bridge=br0/' /lib/systemd/system/systemd-nspawn@.service > /etc/systemd/system/systemd-nspawn@.service",
      "/bin/sed -i 's/^#*SystemMaxUse=.*$/SystemMaxUse=32M/' /etc/systemd/journald.conf",
      "rm /etc/ssh/ssh_host_*",
      "/bin/chmod 755 /usr/local/bin/register-barley-seed",
      "/bin/chmod 755 /usr/local/bin/zap-disk",
      "/bin/chmod 755 /usr/local/bin/attach-disk",
      "/bin/systemctl enable register-barley-seed ssh-host-key",
      "/usr/bin/install -d -m 700 /root/.ssh",
    ]
  }

  provisioner "file" {
    source = "authorized_keys"
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
