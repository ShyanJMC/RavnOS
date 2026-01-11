#!/bin/bash
set -e
kernel=kernel8.img

if [ $# -eq 0 ]
  then
    echo "No arguments supplied"
    echo -e "Arguments/Supported boards;\n rpi4\n rpi5"
    exit
fi

echo "Cleaning temp files and backuping kernel8.img file"
cargo clean 2>/dev/null >/dev/null
if [ -f $kernel ]; then
	mv $kernel $kernel.old
fi
	
echo "Setting rust toolchain"
rustup default nightly
echo "Verifing target and adding if not exist"
rustup target add aarch64-unknown-none-softfloat
	
echo "Compiling"
board=""
cpuboard=""
firmware_hint=""
qemu_hint=""

if [ "$1" == "rpi4" ]; then
	board="bsp_rpi4"
	cpuboard="cortex-a72"
	firmware_hint="firmware/raspberry_pi-4/"
	qemu_hint="[3] Test locally with; qemu-system-aarch64 -M raspi4b -cpu cortex-a72 -smp 4 -m 2G -kernel kernel8.img -device loader,file=firmware/raspberry_pi-4/bcm2711-rpi-4-b.dtb,addr=0x000000000000033c -display none -serial stdio "
elif [ "$1" == "rpi5" ]; then
	board="bsp_rpi5"
	cpuboard="cortex-a76"
	firmware_hint="firmware/raspberry_pi-5/"
	qemu_hint="[3] QEMU for Raspberry Pi 5 is not yet supported upstream; deploy on physical hardware."
else
	echo "Unsupported board '$1'. Supported boards: rpi4 rpi5"
	exit 1
fi

libpath=$(pwd)/src/bsp/raspberrypi
cargo rustc --target aarch64-unknown-none-softfloat --features $board --release -- -C target-cpu=$cpuboard -C link-arg=--library-path=$libpath -C link-arg=--script=$libpath/kernel.ld 2>logs/kernel-error_output.log > logs/kernel-output.log
	
if [ $? -eq 0 ];then

	echo "Copying object and stripping"
	mkdir cargo-tmp
	CARGO_TARGET_DIR=cargo-tmp/ cargo install cargo-binutils >logs/binutils-output.log 2>logs/bintuils-error_output.log
	if [ $? -ne 0 ]; then
		echo "Error building and installing cargo-bintuils. See logs directory."
		exit 1
	fi
	rm -rf cargo-tmp
	
	rustup component add llvm-tools
	rust-objcopy --strip-all -O binary ../target/aarch64-unknown-none-softfloat/release/ravnos_kernel $kernel

	echo "Kernel details;" && ls -lh $kernel

	echo "Creating in ../target/doc the documentation"
	cargo doc --target aarch64-unknown-none-softfloat >/dev/null 2>/dev/null
	echo -e "[0] Insert your SDCard\n[1] Copy kernel8.img and files in ${firmware_hint} into your SDCard.\n[2] Unplug it and boot"
	echo -e "$qemu_hint"

else 
	echo "Failed compiling kernel, verify logs in logs/kernel-*.log file."
fi

echo "Restaring rust toolchain to stable"
rustup default stable
