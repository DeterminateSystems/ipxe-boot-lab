# ipxe-boot-lab

## Usage

Using the provided `shell.nix`, open up one terminal and run:

```console
$ nix-shell
nix-shell$ foreman start
```

This will run everything that the VM tool relies on for isolated networking and
PXE booting (amongst other things).

Then, open another terminal and run:

```console
$ nix-shell
nix-shell$ ./enter.sh
nix-shell$ cd vm
nix-shell$ cargo run -- /path/to//nixos-install-equinix-metal/metadata-examples/c3.medium.x86.json
```

> **NOTE**: At this moment, UEFI-enabled systems do not boot properly.

This will spawn a tmux session with the QEMU monitor (`mon0`) as the first tab,
and the remaining serial ports in an undefined order. Currently, `ttyS0` is used
for PXE boot output, and `ttyS1` is used for the VM's interactive console. To
exit, you have two options:

1. Issue the `poweroff` command from the VM's interactive console
1. Issue the `q` or `quit` command to QEMU's monitor

> **NOTE**: Assumes nix-netboot-serve is checked out to ../nix-netboot-serve and
> probably other stuff too
