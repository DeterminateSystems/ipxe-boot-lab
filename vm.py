#!/usr/bin/env nix-shell
#!nix-shell -p python3 -i python3
import tempfile
import os.path
import sys
import json
import subprocess
import shutil


def interfaces(interfaces: list[dict[str, str]]) -> list[str]:
    args = []
    ctr = 0

    for iface in interfaces:
        netdev = f"netdev{ctr}"
        mac = iface["mac"]

        args += [
            "-netdev",
            f"tap,id={netdev},script=./network-up.sh",
            "-device",
            f"virtio-net-pci,netdev={netdev},mac={mac}",
        ]

        ctr += 1

    return args


def drives(
    drives: list[dict[str, str]], temp_dir: tempfile.TemporaryDirectory, fname: str
) -> list[str]:
    args = ["-device", "virtio-scsi-pci,id=scsi0"]

    for drive in drives:
        # qemu-img doesn't like the "B" suffix (e.g. in GB)
        size = drive["size"].rstrip("B")
        count = drive["count"]
        ty = drive["type"]
        category = drive["category"]
        ctr = 0

        while ctr < int(count):
            outfile = f"{temp_dir}/{fname}-{category}-{ctr}.img"
            ident = f"{category}{ctr}"
            devident = f"dev{ident}"

            subprocess.run(["qemu-img", "create", "-f", "qcow2", outfile, size])

            args += [
                "-drive",
                f"file={outfile},if=none,id={ident}",
            ]

            if ty == "SSD":
                args += [
                    "-device",
                    f"scsi-hd,drive={ident},id={devident}",
                    "-set",
                    f"device.{devident}.rotation_rate=1",
                ]
            elif ty == "HDD":
                args += ["-device", f"scsi-hd,drive={ident},id={devident}"]
            elif ty == "NVME":
                args += ["-device", f"nvme,drive={ident},serial={ident}"]
            else:
                sys.exit(f"Unknown disk type '{ty}'")

            ctr += 1

    return args


def uefi(uefi: bool, temp_dir: tempfile.TemporaryDirectory) -> list[str]:
    args = []

    if uefi:
        shutil.copy2("OVMF_CODE.fd", str(temp_dir))
        shutil.copy2("OVMF_VARS.fd", str(temp_dir))
        args += [
            "-drive",
            f"if=pflash,format=raw,file={temp_dir}/OVMF_CODE.fd,readonly=on",
            "-drive",
            f"if=pflash,format=raw,file={temp_dir}/OVMF_VARS.fd",
        ]

    return args


def metadata(f: str, temp_dir: tempfile.TemporaryDirectory) -> list[str]:
    metadata_api_dir = f"{temp_dir}/metadata-api"
    os.makedirs(metadata_api_dir, exist_ok=True)
    shutil.copy2(f, f"{metadata_api_dir}/metadata.json")

    # todo: userdata, customdata support
    return [
        "-virtfs",
        f"local,id=metadata_api_stub,security_model=none,path={metadata_api_dir},readonly=on,mount_tag=metadata_api_stub",
    ]


def main():
    if len(sys.argv) >= 2 and os.path.isfile(sys.argv[1]):
        f = sys.argv[1]
    else:
        sys.exit("File wasn't provided or didn't exist")

    fname = os.path.splitext(os.path.basename(os.path.realpath(f)))[0]
    doc = json.load(open(f))

    with tempfile.TemporaryDirectory() as temp_dir:
        args = [
            "qemu-kvm",
            "-machine",
            "q35,smm=on",
            "-m",
            "16G",
            "-cpu",
            "max",
            "-serial",
            "null",
            "-serial",
            "mon:stdio",
            "-msg",
            "timestamp=on",
            "-nographic",
        ]
        args += metadata(f, temp_dir)
        args += interfaces(doc["network"]["interfaces"])
        args += drives(doc["specs"]["drives"], temp_dir, fname)
        args += uefi(doc["specs"]["features"]["uefi"], temp_dir)

        subprocess.run(args)


if __name__ == "__main__":
    main()
