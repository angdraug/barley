#!/bin/sh
sed -e 's/^DHCP=yes/DHCP=yes\nIPv6AcceptRA=no/' \
    -e 's/^LinkLocalAddressing=.*$/LinkLocalAddressing=ipv4/' \
    < /lib/systemd/network/80-container-host0.network \
    > /etc/systemd/network/80-container-host0.network
