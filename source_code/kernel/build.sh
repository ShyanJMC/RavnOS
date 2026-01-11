#!/bin/bash
set -e
kernel=kernel8.img

if [ $# -eq 0 ]
  then
    echo "No arguments supplied"
    echo -e "Usage: bash build.sh <board> [--features <comma separated feature list>]"
    echo -e "Supported boards;\n rpi4\n rpi5\n qemu"
    exit
fi

target_board="$1"
shift

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

if [ "$target_board" == "rpi4" ]; then
	board="bsp_rpi4"
	cpuboard="cortex-a72"
	firmware_hint="firmware/raspberry_pi-4/"
	qemu_hint="[3] Test locally with; qemu-system-aarch64 -M raspi4b -cpu cortex-a72 -smp 4 -m 2G -kernel kernel8.img -device loader,file=firmware/raspberry_pi-4/bcm2711-rpi-4-b.dtb,addr=0x000000000000033c -display none -serial stdio "
elif [ "$target_board" == "rpi5" ]; then
	board="bsp_rpi5"
	cpuboard="cortex-a76"
	firmware_hint="firmware/raspberry_pi-5/"
	qemu_hint="[3] QEMU for Raspberry Pi 5 is not yet supported upstream; deploy on physical hardware."
elif [ "$target_board" == "qemu" ]; then
	board="bsp_qemu"
	cpuboard="cortex-a72"
	firmware_hint="firmware/raspberry_pi-4/"
	qemu_hint="[3] Test locally with; qemu-system-aarch64 -M raspi4b -cpu cortex-a72 -smp 4 -m 2G -kernel kernel8.img -device loader,file=firmware/raspberry_pi-4/bcm2711-rpi-4-b.dtb,addr=0x000000000000033c -display none -serial stdio\n[4] The build also enables PSCI helpers so you can run: qemu-system-aarch64 -M virt -cpu cortex-a72 -smp 4 -m 2G -kernel kernel8.img -serial stdio -display none"
else
	echo "Unsupported board '$target_board'. Supported boards: rpi4 rpi5 qemu"
	exit 1
fi

extra_features=""
while [ $# -gt 0 ]; do
	case "$1" in
		--features)
			shift
			if [ $# -eq 0 ]; then
				echo "Missing feature list after --features"
				exit 1
			fi
			extra_features="$1"
			;;
		"")
			;;
		*)
			echo "Unknown option: $1"
			exit 1
			;;
	esac
	shift
done

if [ -n "$extra_features" ]; then
	feature_list="${board},${extra_features}"
else
	feature_list="${board}"
fi

if [ "$target_board" == "qemu" ]; then
	if [[ "$feature_list" == *"qemu_psci"* ]]; then
		echo "[build] qemu_psci already requested via CLI; leaving feature list untouched"
	else
		if [ -n "$feature_list" ]; then
			feature_list="${feature_list},qemu_psci"
			else
				feature_list="qemu_psci"
			fi
		echo "[build] Enabling qemu_psci feature for QEMU target"
	fi
fi

libpath=$(pwd)/src/bsp/raspberrypi
cargo rustc --target aarch64-unknown-none-softfloat --features "$feature_list" --release -- -C target-cpu=$cpuboard -C link-arg=--library-path=$libpath -C link-arg=--script=$libpath/kernel.ld 2>logs/kernel-error_output.log > logs/kernel-output.log
	
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
