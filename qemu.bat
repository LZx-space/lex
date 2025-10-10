@echo off
chcp 65001

Set QEMU=C:/Users/LZx/qemu/qemu-system-riscv64.exe
Set MACH=virt
Set CPU=rv64
Set CPUS=4
Set MEM=128M
Set DRIVE=hdd.dsk
Set OUT=os.elf

:: C:/Users/LZx/qemu/qemu-system-riscv64.exe -machine virt -nographic -bios none -device loader,file=target/riscv64gc-unknown-none-elf/release/lex,addr=0x80000000
:: %QEMU% -machine %MACH% -cpu %CPU% -smp %CPUS% -m %MEM%  -nographic -serial mon:stdio -bios none -kernel %OUT% -drive if=none,format=raw,file=%DRIVE%,id=foo -device virtio-blk-device,scsi=off,drive=foo
set CMD=%QEMU% -machine %MACH% -cpu %CPU% -smp %CPUS% -m %MEM% -nographic -serial mon:stdio -bios none -device loader,file=target/riscv64gc-unknown-none-elf/release/lex,addr=0x80000000
echo "运行命令：%CMD%"
%CMD%

pause