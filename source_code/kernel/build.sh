#!/bin/bash

if [ $# -eq 0 ]
  then
    echo "No arguments supplied"
    echo -e "Arguments/Supported boards;\n rpi4"
    exit
fi


if [ $1 == "rpi4" ]; then
	kernel=kernel8.img
	board="bsp_rpi4"
	cpuboard="cortex-a72"
	libpath=$(pwd)/src/bsp/raspberrypi	
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
cargo rustc --target aarch64-unknown-none-softfloat --features $board --release -- -C target-cpu=$cpuboard -C link-arg=--library-path=$libpath -C link-arg=--script=$libpath/kernel.ld 
	
if [ $? -eq 0 ];then
	echo "Copy object and stripping"
	rust-objcopy --strip-all -O binary ../target/aarch64-unknown-none-softfloat/release/ravnos_kernel $kernel
	ls -lh $kernel
	echo "Creating in ../target/doc the documentation"
	cargo doc --target aarch64-unknown-none-softfloat
	echo -e "[0] Insert your SDCard\n[1] Copy kernel8.img and files in firmware/raspberry_pi-4/ into your SDCard.\n[2] Unplug it and boot"
fi

