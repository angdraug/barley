source "nspawn" "cryptpad" {
  clone = "base"
}

build {
  sources = ["source.nspawn.cryptpad"]

  provisioner "apt" {
    packages = ["git", "nginx-light", "npm"]
  }

  provisioner "shell" {
    inline = [
      "npm install -g bower",
      "adduser --disabled-login --gecos CryptPad cryptpad",
    ]
  }

  provisioner "shell" {
    script = "packer/cryptpad.sh"
    execute_command = "su - cryptpad -c '/bin/sh {{ .Path }}'"
  }

  provisioner "file" {
    source = "packer/cryptpad.config.js"
    destination = "/home/cryptpad/cryptpad/config/config.js"
  }

  provisioner "file" {
    source = "packer/cryptpad.service"
    destination = "/etc/systemd/system/cryptpad.service"
  }

  provisioner "file" {
    source = "packer/cryptpad.nginx.conf"
    destination = "/etc/nginx/sites-available/cryptpad"
  }

  provisioner "shell" {
    inline = [
      "install -o www-data -g www-data -d /var/lib/cryptpad",
      "rm /etc/nginx/sites-enabled/default",
      "ln -s /etc/nginx/sites-available/cryptpad /etc/nginx/sites-enabled/",
      "systemctl daemon-reload",
      "systemctl enable cryptpad",
      "apt-get -y autoremove git",
    ]
  }

  provisioner "shell" {
    script = "no-ipv6.sh"
  }

  post-processors {
    post-processor "shell-local" {
      inline = [
        "tar --zstd -C /var/lib/machines/cryptpad -cf cryptpad.tar.zst .",
      ]
    }

    post-processor "artifice" {
      files = ["cryptpad.tar.zst"]
    }
  }
}
