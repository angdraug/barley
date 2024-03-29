# Barley

Barley is an ephemeral image-based bare metal provisioning system.

- A **provisioning** system installs a specific version and configuration of an
  operating system to multiple machines.

- **Bare metal** is physical hardware, as opposed to virtual machines or
  containers.

- **Image-based** provisioning uses pre-generated disk images instead of
  running an OS installer on every target host.

- **Ephemeral** means the OS is not preserved between reboots.

Barley build script combines a minimum base Debian system with Linux kernel and
systemd-nspawn into a Seed initramfs image. Barley Sower serves the Seed image
using PXE network boot protocol. Barley Seed skips the pivot step of Linux boot
process and runs systemd and the rest of the OS directly from rootfs in RAM.

Every reboot provisions the latest OS image, and the entire boot/provision
sequence takes approximately 10s from initiating network boot to accepting SSH
connections and launching containers.

## Quick Start

```sh
make release
sudo make
sudo make install

sow new field-1
sow import sower.tar.zst
sow start --ca --local sower

sudo mkdir /etc/qemu
echo allow br0 > /etc/qemu/bridge.conf
sudo cp qemu-seed.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl start qemu-seed
journalctl -f -u qemu-seed

sudo make postgres.tar.zst
ssh seed-1 'zstdcat | machinectl import-tar - postgres-1' < postgres.tar.zst
```

## Setup

Prerequisites:
- [Systemd](https://systemd.io/) (systemd-container, systemd-networkd, systemd-resolved)
- [Rust](https://www.rust-lang.org/)
- [Packer](https://packer.io/)
- [packer-builder-nspawn](https://git.sr.ht/~angdraug/packer-builder-nspawn)
- [packer-provisioner-apt](https://git.sr.ht/~angdraug/packer-provisioner-apt)
- zstd
- gnutls-bin
- intel-microcode
- (optional) qemu-system-x86

When installing Packer from Debian, use `apt-get --no-install-recommends` to
prevent it from also installing Docker as a dependency.

When building the packer-builder-nspawn and packer-provisioner-apt plugins from
source, symlink them into your working directory so that Packer can find them.

## Network

Seed host binds all its physical Ethernet interfaces to a bridge named br0. To
connect a new container to br0, Barley `sow start` command creates a following
[systemd.nspawn(5)](https://www.freedesktop.org/software/systemd/man/systemd.nspawn.html)
container configuration file under `/etc/systemd/nspawn`:

```systemd
[Network]
Bridge=br0
```

You can use the same
[systemd-networkd](https://www.freedesktop.org/software/systemd/man/systemd-networkd.html)
configs that are used by Seeds to set up br0 for local containers:

```sh
sudo cp network/* /etc/systemd/network/
sudo networkctl reload
sudo networkctl up br0
sed 's/--network-veth/--network-bridge=br0/' \
  /lib/systemd/system/systemd-nspawn@.service \
  > /etc/systemd/system/systemd-nspawn@.service
```

Sower container expects to be directly connected to a network that already has
a DHCP server (e.g. the router on a typical home network). Sower obtains its
own IP configuration from DHCP and leaves it up to the existing DHCP server to
allocate IP addresses to PXE clients.

## SSH Access to Seeds

Seed root account is passwordless and the only way to access a Seed host is by
authenticating with an SSH key. When you create a new field with `sow new`,
Barley will use your `~/.ssh/id_ed25519.pub` (or any other public key you
specify with the `-k` option) as the admin key. When a Seed registers with
Sower, it will be provisioned with an `authorized_keys` file that allows the
field admin key to both login directly and to sign other keys.

With that, you can generate short-lived passwordless SSH keys for use with
automation:

```sh
ssh-keygen -C barley -f ~/.ssh/id_barley -N '' -t ed25519
ssh-keygen -I barley -s ~/.ssh/id_ed25519 -V +1d ~/.ssh/id_barley
ssh -i ~/.ssh/id_barley seed-1
```

## Persistent Storage

While the host OS managed by Barley and the container code managed by
systemd-nspawn remain ephemeral, your data doesn't have to be.

Seed image includes basic tools to manage persistent storage:

- `zap-disk` creates a single GPT partition spanning the entire block device,
  formats that partition to LUKS with a randomly generated 512-byte key file,
  and creates an LVM volume group that you can slice into logical volumes.

  It is up to you to safely backup the key file and redeploy it after rebooting
  the Seed. If you lose the key you lose all data that was encrypted with it.
  Always encrypt the key before writing it to any persistent storage:

  ```sh
  ssh seed-1 cat /root/luks-key-sda | gpg -e -o luks-key-sda.gpg
  ```

- `attach-disk` finds a logical volume with the same name as the container,
  creates and formats a 4GB ext4 file system if such volume doesn't exist,
  mounts it to the specified path inside the container, and updates ownership
  to the specified user within the container namespace. You can override file
  system size and type like this:

  ```sh
  SIZE=32G FS=xfs attach-disk postgres-1 /var/lib/postgresql postgres
  ```

  By default, systemd-nspawn allocates uid namespaces based on consistent hash
  of container name. This means that, unless you luck into a hash collision
  between container names on the same Seed, file ownership in the persistent
  volume is going to automatically remain in sync with the container user
  namespace across container restarts and Seed host reboots.

Remember that data on persistent storage still needs backups.

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
your own time spent on running the infrastructre often outweighs the cost of
paying public cloud providers 5-20x of what the same compute and storage
capacity would have cost you in your own hardware and electricity.

Barley attempts to close this gap and make bare metal provisioning as
effortless at the scale of a home lab as it is at the scale of a datacenter.
Where enterprise grade and web scale solutions overwhelm you with configuration
variations and micro-optimizations, Barley takes away your options until what's
left is simple enough to just work.

## Implementation Choices

In a continuously deployed cloud native environment, you more often need to
update your OS than you need to reboot your servers. What's the point of
writing OS image to persistent storage if you'll need to replace it more than
once by the time you need to reboot?

Instead, Barley Seed runs directly out of
[rootfs](https://www.kernel.org/doc/Documentation/filesystems/ramfs-rootfs-initramfs.txt).
In modern Linux, rootfs is based on
[tmpfs](https://www.kernel.org/doc/Documentation/filesystems/ramfs-rootfs-initramfs.txt),
an in-memory filesystem that is effectively just page cache without the backing
block storage. On a busy system, most of code and data from your OS and your
container images would end up in the page cache anyway, so the real memory cost
of running the OS without a backing block device is smaller than the 650MB
taken up by the unpacked Seed image.

Barley leaves room for some memory optimizations that bring too much complexity
to be worth the trouble for most users:

- Set up a CI pipeline to compile your own kernels with just the drivers for
  your hardware, and use those when building Seed images. You will save up to
  250MB per host (that's only 3% of 8GB, use distro kernels you miser).

- Create and mount an [encrypted swap
  partition](https://wiki.archlinux.org/index.php/Dm-crypt/Swap_encryption) on
  persistent storage. Savings depend on how much rarely used data is baked into
  your container images, the costs include taking IOPS away from your databases
  and increasing your SSD burn rate.

- Setting up [ksmtuned](https://www.kernel.org/doc/Documentation/vm/ksm.txt)
  can save a good percentage of memory if you run many small containers
  (packer-builder-nspawn base image that is likely to be shared between the
  Seed OS and all your containers unpacks to 200MB). The costs are a small
  fraction of CPU and a potential side channel attack surface.

The quest for simplicity above all also dictates Barley's choice of
systemd-nspawn as container runtime and APT as the preferred package manager.
The fundamental building bricks of a mature Linux distro are most readily
available, easiest to learn, and least likely to break.

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
