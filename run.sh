[ -z $OVMF_DIR ] && OVMF_DIR=/usr/share/edk2-ovmf

TARGET=x86_64-unknown-uefi

CARGO_ARGS="$CARGO_ARGS --target $TARGET"
BUILD_TYPE="debug"

if [ "$1" = "release" ]
then
    CARGO_ARGS="$CARGO_ARGS --release"
    BUILD_TYPE="release"
fi


mkdir -p $PWD/esp/EFI/Boot


cargo xbuild $CARGO_ARGS &&
cp $PWD/target/$TARGET/$BUILD_TYPE/qni-os.efi $PWD/esp/EFI/Boot/BootX64.efi &&

qemu-system-x86_64 \
    -nodefaults \
    -vga std -serial stdio \
    -machine q35,accel=kvm:tcg -m 128M \
    -drive if=pflash,format=raw,file="$OVMF_DIR/OVMF_CODE.fd",readonly=on \
    -drive if=pflash,format=raw,file="$OVMF_DIR/OVMF_VARS.fd",readonly=on \
    -drive format=raw,file=fat:rw:$PWD/esp

