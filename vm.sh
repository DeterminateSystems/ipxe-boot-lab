

#url=http://127.0.0.1:3030/boot/g1y8607jmwnxgwhb92jgh8s8j8ybvjrm-nixos-system-m1small-21.11pre307912.fe01052444c/netboot.ipxe
#url=http://127.0.0.1:3030/dispatch/profile/m1.small
url=http://127.0.0.1:3030/dispatch/configuration/small
url=http://192.168.0.1:3030/dispatch/configuration/small

set -ux

scratch=$(mktemp -d -t tmp.XXXXXXXXXX)
function finish {
  rm -rf "$scratch"
}
trap finish EXIT

jq -cn '$ARGS.named' --arg  rows "$(tput lines)" --arg cols "$(tput cols)" > "$scratch/stty.json"
cat "$scratch/stty.json"
sleep 5

ssd_args() (
  id=$1
  sizeGb=$2
  qemu-img create -f qcow2 "$scratch/$id" "${sizeGb}G" >&2
  echo -device virtio-scsi-pci,id=scsi0
  echo -drive file="$scratch/$id",if=none,id="${id}"
  echo -device scsi-hd,drive="${id}",id="${id}dev"
  echo -set device."${id}dev".rotation_rate=1
)

qemu-kvm \
  -enable-kvm \
  -m 16G \
  -cpu max \
  -serial mon:stdio \
  -smbios type=11,path="$scratch/stty.json" \
  -netdev tap,id=nd1,script="./network-up.sh" \
  -device virtio-net-pci,netdev=nd1 \
  -netdev tap,id=nd2,script="./network-up.sh" \
  -device virtio-net-pci,netdev=nd2 \
  -msg timestamp=on \
  -virtfs local,id=metadata_api_stub,security_model=none,path=./metadata-api,readonly=on,mount_tag=metadata_api_stub \
  $(ssd_args boot 80) \
  -nographic
