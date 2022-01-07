namespace-setup: ./create.sh
network-setup: ./enter.sh ./network-setup.sh
dnsmasq: ./enter.sh ./dnsmasq.sh
nix-netboot-serve: ./enter.sh ./nix-netboot-serve.sh
namespace-networking: ./namespace-networking.sh
namespace-networking-setup: ./enter.sh ./network-setup-slirp.sh

# Start the tmux daemon manually, otherwise
# tmux may not start up properly.
tmux-daemon: ./enter.sh $TMUX_PATH -D -L ipxe-boot-lab
