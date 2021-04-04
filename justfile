target := "riscv64imac-unknown-none-elf"
mode := "debug"
build-path := "target/" + target + "/" + mode + "/"

bootloader-elf := "../rustsbi/target/" + target + "/debug/rustsbi-qemu"
bootloader-bin := "../rustsbi/target/" + target + "/debug/rustsbi-qemu.bin"
kernel-elf := build-path + "tornado-kernel"
kernel-bin := build-path + "tornado-kernel.bin"
user-elf := build-path + "tornado-user"
user-bin := build-path + "tornado-user.bin"

objdump := "riscv64-unknown-elf-objdump"
gdb := "riscv64-unknown-elf-gdb"
size := "rust-size"

threads := "1"

build:
    @just -f "tornado-kernel/justfile" build

build-user:
    @just -f "tornado-user/justfile" build
    
qemu: build build-user
    @qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios none \
            -device loader,file={{bootloader-bin}},addr=0x80000000 \
            -device loader,file={{kernel-bin}},addr=0x80200000 \
            -device loader,file={{user-bin}},addr=0x87000000 \
            -smp threads={{threads}}

run: build qemu

asm: build
    @{{objdump}} -D {{kernel-elf}} | less

asm-user: build-user
    @{{objdump}} -D {{user-elf}} | less
    
size: build
    @{{size}} -A -x {{kernel-elf}}
    @{{size}} -A -x {{user-elf}} 


debug: build
    @qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios none \
            -device loader,file={{bootloader-bin}},addr=0x80000000 \
            -device loader,file={{kernel-bin}},addr=0x80200000 \
            -device loader,file={{user-bin}},addr=0x87000000 \
            -smp threads={{threads}} \
            -gdb tcp::1234 -S
            
gdb: 
    @{{gdb}} --eval-command="file {{kernel-elf}}" --eval-command="target remote localhost:1234"
