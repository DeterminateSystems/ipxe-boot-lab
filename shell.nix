let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs { };
in
pkgs.mkShell {
  buildInputs = [
    pkgs.binwalk
    pkgs.entr
    pkgs.file
    pkgs.foreman
    pkgs.nftables
    pkgs.qemu
    pkgs.slirp4netns
    pkgs.vim # xxd
    pkgs.ncurses # tput
    pkgs.jq

    pkgs.cargo
    pkgs.rustfmt
    pkgs.rustc
  ];

  OVMF_PATH = pkgs.OVMF.fd;
  TMUX_PATH = "${pkgs.tmux}/bin/tmux";
  SCREEN_PATH = "${pkgs.screen}/bin/screen";
}
