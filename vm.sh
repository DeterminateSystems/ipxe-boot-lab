#!/usr/bin/env bash
set -eux
set -o pipefail

file="${1-}"

if [ -z "$file" ] || [ ! -e "$file" ]
then
  echo "File was empty or didn't exist"
  exit 1
fi

scratch="$(mktemp -d -t tmp.XXXXXXXXXX)"
function finish {
  rm -rf "$scratch"
}
trap finish EXIT

spec="$(jq -cr < "$file")"
filename="${file##*/}"
filename="${filename%.*}"

cmd=()

metadata() {
  metadata_api_dir="$scratch/metadata-api"

  mkdir -p "$metadata_api_dir"
  cp "$file" "$metadata_api_dir/metadata.json"

  # todo: userdata, customdata support

  cat <<EOT
-virtfs local,id=metadata_api_stub,security_model=none,path=$metadata_api_dir,readonly=on,mount_tag=metadata_api_stub
EOT

}

interfaces() {
  ctr=0

  jq -r '.network.interfaces[] | .mac' <<< "$spec" | while read -r mac
  do
    netdev="netdev${ctr}"
    cat <<EOT
-netdev tap,id=$netdev,script=./network-up.sh
-device virtio-net-pci,netdev=$netdev,mac=$mac
EOT
    ctr="$((ctr+1))"
  done
}

drives() {
  cat <<EOT
-device virtio-scsi-pci,id=scsi0
EOT

  jq -cr '.specs.drives[]' <<< "$spec" | while read -r drive
  do
    count="$(jq -r .count <<< "$drive")"
    size="$(jq -r .size <<< "$drive")"
    size="${size%B}" # strip off the B from GB because qemu-img doesn't like that
    ty="$(jq -r .type <<< "$drive")"
    category="$(jq -r .category <<< "$drive")"

    ctr=1
    until [ "$ctr" -gt "$count" ]
    do
      f="${scratch}/${filename}-${category}-${ctr}.img"
      id="${category}${ctr}"
      devid="dev${id}"

      qemu-img create -f qcow2 "$f" "$size" > /dev/null

      cat <<EOT
-drive file=$f,if=none,id=$id
EOT

      case "$ty" in
        SSD)
          cat <<EOT
-device scsi-hd,drive=$id,id=$devid
-set device.$devid.rotation_rate=1
EOT
          ;;
        HDD)
          cat <<EOT
-device scsi-hd,drive=$id,id=$devid
EOT
          ;;
        NVME)
          cat <<EOT
-device nvme,drive=$id,serial=$id
EOT
          ;;
        *)
          echo "Invalid disk type '$ty'" >&2
          exit 1
          ;;
      esac

      ctr="$((ctr+1))"
    done
  done
}

cmd+=(
  'qemu-kvm'
  '-enable-kvm'
  '-m 16G'
  '-cpu max'
  '-serial mon:stdio'
  '-machine q35,smm=on'
  '-msg timestamp=on'
  '-nographic'
  $(metadata)
  $(interfaces)
  $(drives)
)

${cmd[@]}
