{
  pkgs,
  kernel,
  loader,
}:

pkgs.stdenv.mkDerivation {
  pname = "boot-image";
  version = "0.1.0";

  nativeBuildInputs = [
    pkgs.dosfstools # mkfs.fat
    pkgs.mtools # mcopy, mmd
    pkgs.coreutils
  ];

  buildInputs = [
    kernel
    loader
  ];

  dontUnpack = true;

  buildPhase = ''
    IMG=boot.img
    FILES=(
      ${loader}/bin/uefi-loader.efi
      ${kernel}/bin/kernel.elf
      ${../psf/light16.psf}
    )

    # Sum file sizes in bytes
    TOTAL_BYTES=0
    for f in "''${FILES[@]}"; do
      TOTAL_BYTES=$((TOTAL_BYTES + $(stat -c '%s' "$f")))
    done

    SIZE_MB=$(( (TOTAL_BYTES + 1024*1024 - 1) / (1024*1024)))
    if [[ SIZE_MB -le 32 ]]; then
      SIZE_MB=33
    fi

    echo "Creating empty FAT image..."
    truncate -s ''${SIZE_MB}M $IMG
    mkfs.fat -F 32 $IMG

    echo "Creating directory structure..."
    mmd -i $IMG ::/efi
    mmd -i $IMG ::/efi/boot

    echo "Copying loader..."
    mcopy -i $IMG "''${FILES[0]}" ::/efi/boot/bootx64.efi

    echo "Copying kernel..."
    mcopy -i $IMG "''${FILES[1]}" ::/kernel.elf

    echo "Copying font..."
    mcopy -i $IMG "''${FILES[2]}" ::/font.psf

    chmod +w boot.img
  '';

  installPhase = ''
    mkdir -p $out
    cp boot.img $out/
  '';
}
