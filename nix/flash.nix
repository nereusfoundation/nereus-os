{
  pkgs,
  bootimage,
}:

pkgs.writeShellApplication {
  name = "nereus-flash";
  runtimeInputs = with pkgs; [
    popsicle
  ];

  text = ''
    set -euo pipefail
    echo "Select disk to write ISO to (this will erase EVERYTHING!):"
    lsblk
    read -r mydisk
    sudo popsicle ${bootimage}/boot.img "$mydisk"
  '';

}
