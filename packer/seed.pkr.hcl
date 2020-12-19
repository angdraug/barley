source "nspawn" "seed" {
  clone = "base"
}

build {
  sources = ["source.nspawn.seed"]

  provisioner "apt" {
    packages = [
      # required
      "linux-image-amd64", "iproute2", "openssh-server", "curl",

      # troubleshooting tools (optional)
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
    source = "register-barley-seed"
    destination = "/usr/local/bin/register-barley-seed"
  }

  provisioner "file" {
    sources = ["register-barley-seed.service", "ssh-host-key.service"]
    destination = "/etc/systemd/system/"
  }

  provisioner "shell" {
    inline = [
      "/bin/sed 's/--network-veth/--network-bridge=br0/' /lib/systemd/system/systemd-nspawn@.service > /etc/systemd/system/systemd-nspawn@.service",
      "/bin/sed -i 's/^#*SystemMaxUse=.*$/SystemMaxUse=32M/' /etc/systemd/journald.conf",
      "rm /etc/ssh/ssh_host_*",
      "echo HostKey /etc/ssh/ssh_host_ed25519_key > /etc/ssh/sshd_config.d/host_key.conf",
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
