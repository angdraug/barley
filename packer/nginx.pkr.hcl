source "nspawn" "nginx" {
  clone = "base"
}

build {
  sources = ["source.nspawn.nginx"]

  provisioner "apt" {
    packages = ["nginx-light"]
  }

  provisioner "shell" {
    inline = [
      "systemctl disable nginx",
      "rm /etc/nginx/sites-enabled/default",
      "rm /var/www/html/index.nginx-debian.html",
    ]
  }

  post-processors {
    post-processor "shell-local" {
      inline = [
        "tar --zstd -C /var/lib/machines/nginx -cf nginx.tar.zst .",
        "machinectl remove nginx",
      ]
    }

    post-processor "artifice" {
      files = ["nginx.tar.zst"]
    }
  }
}
