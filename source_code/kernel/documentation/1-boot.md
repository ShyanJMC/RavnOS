# Boot

In ARM architecture the boot process is not standarized, each computer
have a specific one.

In Rasberry Pi the internal firmware load the file "config.txt" with 
the hardware configurations on it.

RavnOS requires; 64 bits enabled,
the DTB (Device Tree Base) file in 0x000000000000033c region, and UART enabled.

```text
arm_64bit=1
kernel=kernel8.img
kernel_address=0x80000

enable_uart=1
device_tree=bcm2711-rpi-4-b.dtb
device_tree_address=0x000000000000033c
```

You can test the kernel with qemu using this command;

> qemu-system-aarch64 -M raspi4b -cpu cortex-a72 -smp 4 -m 2G -kernel kernel8.img -device loader,file=firmware/raspberry_pi-4/bcm2711-rpi-4-b.dtb,addr=0x000000000000033c -display none -serial stdio 

As you can verify above, it specify the DTB file with specific address.

The value of 0x80000 for kernel location in physical memory is the standard.

## CPU and Simultaneous Proccesing 

By default RavnOS start (main.rs -> kernel_init() ) as single core, then
start function "kernel_main()" to get cores number and start them ( cpu/mod.rs -> mod.rs -> start_cores() ).

There are two ways in ARM64 to start cores; spin-table and PSCI. Each CPU works using events, by default
only one core starts (the core zero, called "Boot Strap Processor"), 
the rest have the event "WFE" (Wait for Event; basically are IDLE).

- Spin Table

  The BSP (the core zero) reserve a specific memory region, which will be used as Spin Table.

  Each secundary CPU core will have an entry in the Spin Table, and each entry have a specific 
  memory address. When a secundary CPU core starts, read that specific memory address for him 
  and start processing what is on it memory location (because of that, each memory address entry
  in the Spin Table must be the program start that you want execute).

  The way to communicate between cores ("start the execution", "stop the execution", etc) is using
  the MAILBOX (and the sub registries of it) to send events and controll the execution.
