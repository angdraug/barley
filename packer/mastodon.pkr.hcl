source "nspawn" "mastodon" {
  clone = "base"
}

build {
  sources = ["source.nspawn.mastodon"]

  provisioner "apt" {
    packages = [
      "build-essential", "bundler", "ffmpeg", "file", "git", "imagemagick",
      "libicu-dev", "libidn11-dev", "libpq-dev", "libjemalloc-dev",
      "zlib1g-dev", "libgdbm-dev", "libgmp-dev", "libssl-dev", "libyaml-0-2",
      "libreadline8", "python3", "shared-mime-info", "ruby3.0", "ruby3.0-dev",
      "wget", "whois", "yarnpkg",
    ]
  }

  provisioner "shell" {
    inline = [
      "ln -fs /usr/share/zoneinfo/Etc/UTC /etc/localtime",
      "ln -fs ruby3.0 /usr/bin/ruby",
      "ln -fs yarnpkg /usr/bin/yarn",
      "groupadd -g 991 mastodon",
      "useradd -l -u 991 -g 991 -d /opt/mastodon mastodon",
      "install -m 755 -o mastodon -g mastodon -d /opt/mastodon",
      "ln -s /opt/mastodon /mastodon",
    ]
  }

  provisioner "shell" {
    script = "packer/mastodon.sh"
    execute_command = "su - mastodon -c '/bin/sh -eux {{ .Path }}'"
  }

  provisioner "shell" {
    inline = [
      "apt-get -y autoremove git",
    ]
  }

  provisioner "shell" {
    script = "no-ipv6.sh"
  }

  post-processors {
    post-processor "shell-local" {
      inline = [
        "tar --zstd -C /var/lib/machines/mastodon -cf mastodon.tar.zst .",
        "machinectl remove mastodon",
      ]
    }

    post-processor "artifice" {
      files = ["mastodon.tar.zst"]
    }
  }
}
