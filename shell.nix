let pkgs = import <nixpkgs> { }; in
pkgs.mkShell {
  buildInputs = [
    pkgs.binwalk
    pkgs.cargo
    pkgs.entr
    pkgs.file
    pkgs.foreman
    pkgs.nftables
    pkgs.qemu
    pkgs.rustfmt
    pkgs.slirp4netns
    pkgs.vim # xxd
    pkgs.ncurses # tput
    pkgs.jq
    pkgs.black
  ];

  shellHook = ''
    cp -f ${pkgs.OVMF.fd}/FV/OVMF_*.fd .
    chmod 644 ./OVMF_VARS.fd
  '';
}
