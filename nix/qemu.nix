{
  pkgs,
  bootimage,
}:

pkgs.writeShellApplication {
  name = "nereus-vm";
  runtimeInputs = with pkgs; [
    qemu
  ];

  text = ''
    set -euo pipefail
    OVMF="${pkgs.OVMF.fd}/FV"

    # writeable location
    IMG="$(mktemp -t nereus-boot.XXXXXX.img)"
    cp "${bootimage}/boot.img" "$IMG"

    exec qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF"/OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF"/OVMF_VARS.fd \
    -drive format=raw,file="$IMG" \
    -serial stdio \
    -d int \
    -D qemu.log \
    -no-reboot \
    -m 512M
  '';

}
