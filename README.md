# Barley

Barley is an ephemeral image-based bare metal provisioning system.

A *provisioning* system installs a specific version and configuration of an
operating system to multiple machines.

*Bare metal* is physical hardware, as opposed to virtual machines or
containers.

*Image-based* provisioning uses pre-generated disk images instead of running an
OS installer on every target host.

*Ephemeral* means the OS is not preserved between reboots.

Barley build script combines a minimum base Debian system with Linux kernel and
systemd-nspawn into a Seed initramfs image. Barley Sower serves the Seed image
using PXE network boot protocol. Barley Seed skips the pivot step of Linux boot
process and runs systemd and the rest of the OS directly from rootfs.

Every reboot provisions the latest OS image, and the entire boot/provision
sequence takes approximately 10s from initiating network boot to accepting SSH
connections and launching containers.

## Quick Start

```
ln -s ~/.ssh/id_ed25519.pub .
packer build -only '*.seed' .
packer build -only '*.sower' .
echo -e '[Network]\nBridge=br0' > /etc/systemd/nspawn/sower.nspawn
machinectl start sower
./test-seed
```

## Setup

Prerequisites:
- [Packer](https://packer.io/)
- [packer-builder-nspawn](https://git.sr.ht/~angdraug/packer-builder-nspawn)
- [packer-provisioner-apt](https://git.sr.ht/~angdraug/packer-provisioner-apt)
- intel-microcode
- (optional) qemu-system-x86

When installing Packer from Debian, use `apt-get --no-install-recommends` to
prevent it from also installing Docker as a dependency.

After building the packer-builder-nspawn and packer-provisioner-apt plugins,
symlink them into your working directory so that Packer can find them.

Before building the Seed image, symlink your public SSH key into your working
directory. Seed root account is passwordless and the only way to access a Seed
host is by authenticating with that SSH key.

Seed starts with all its physical Ethernet interfaces bound to a Linux bridge
named br0. For any container that requires public network, put a
[systemd.nspawn(5)](https://www.freedesktop.org/software/systemd/man/systemd.nspawn.html)
file like below under `/etc/systemd/nspawn`:

```
[Network]
Bridge=br0
```

Sower expects to be directly connected to a network that already has a DHCP
server. You can use the same Linux bridge setup as described above to achieve
that. Sower obtains its own IP configuration from DHCP and leaves it up to the
existing DHCP server to allocate IP addresses to PXE clients.

## Motivation

Barley is rooted in the first principal tenet of the Earthseed doctrine
introduced in Octavia Butler's Parable of Sower: God is Change.

Cloud native computing is all about observing and shaping change. Package
software into containers to make change atomic. Loosely couple microservices to
make change granular. Measure and monitor everything to make change observable.
Automate deployment and scaling to make change immediate.

Unfortunately, democratization of cloud native software deployment patterns
came at the cost of centralizing cloud infrastructure into town sized data
halls owned by a handful of large corporations.

Building the under-cloud on your own hardware doesn't scale down: the cost of
your own time spent on running the infrastructre quickly the cost of paying
public cloud providers 5-20x of what the same compute and storage capacity
would have cost you in hardware and electricity.

Barley attempts to close this gap and make bare metal provisioning as
effortless at the scale of a home lab as it is at the scale of a datacenter.
Where enterprise grade and web scale solutions overwhelm you with configuration
variations and micro-optimizations, Barley takes away your options until what's
left is simple enough to just work.

In a continuously deployed cloud native environment, you more often need to
update your OS than you need to reboot your servers. So what's the point of
writing OS image to persistent storage if you'll need to replace it more than
once by the time you reboot?

At datacenter scale, the labor cost of compiling your own kernels with just the
drivers you need to save 200MB of RAM per host might pay off. At home, you're
better off standing on the shoulders of giants and using distro kernels.

Same goes for Barley's choice of systemd-nspawn as container runtime and APT
for package management. The fundamental building bricks of a mature Linux
distro are most readily available, easiest to learn, and least likely to break.

## Copying

Copyright (c) 2020  Dmitry Borodaenko <angdraug@debian.org>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
