# RavnOS
RanvOS, from norwegian; raven,  is a operative system programmed in Rust.

Aims to be; minimal, fast, secure and modern ( this maybe you know as; state-of-art ).

## Target
The target of this project is not do the next Linux, GNU HURD or Redox. 

Is my personal project to make from scratch an operative system with special focus in minimalism, portability, cyber security and wherever I need in the future.

Take this project as a hobby, nothing professional.

I prefer build my programs as statically linked, with native CPU support and without debug information. Because of that I recommend set this configuration in " ~/.cargo/config.toml ":

> \[build]
> 
> jobs = 20
> 
> rustc = "rustc"
> 
> rustflags = ["-C","opt-level=2","-C","debuginfo=0","-C", "target-feature=+crt-static","-C","target-cpu=native"]

## Standard

When I writte RavnOS I just put this topics as standard for development, distribution and use;

1. No external crates allowed (crates from internet).

2. The musl standard c-lib is used for compile.

3. All must be compiled as static binary. Becuase of this musl is the best option.

## Note
I like do this project thiking; 

> if you are in the middle of mountain (or in the middle of zombie apocalypse) without Starlink or any other internet connection, how you will build this?

Because of that, and one fun challenge, is not add dependencies that need internet, all must be local. 

## Requirements 
- Rustc
- A terminal
- Cargo with the target toolchain you want/need: This is not mandatory but will help you to build with one just command.
- Musl target: This is not mandatory, but I recommend it to build as static.

## Build

There are two ways for build RavnOS;

- Cargo

```rust
cargo build --release --target [x86_64/arm64/etc]-unknown-[linux/windows/etc]-musl
```

All binaries will be in "target/\[TARGET]/release".

If you have space requeriments, do "strip" to the final binaries. This is because even with "--release" target I still found debug symbols in the final binary.

- Rustc

For each RavnOS's lib you must first build them as object file and then can be used inside compilation process;

```rust
rustc --crate-type=rlib --crate-name libconfarg [PATH_TO_LIBCONFARG]/src/lib.rs -o libconfarg.rlib
rustc --crate-type=rlib --crate-name libstream [PATH_TO_LIBSTREAM]/src/lib.rs -o libstream.rlib
rustc --crate-type=rlib --crate-name libfile [PATH_TO_LIBFILE]/src/lib.rs -o libfile.rlib
```

Then you can link into the binary build;

```rust
rustc --target=[x86_64/arm64/etc]-unknown-[linux/windows/etc]-musl -C opt-level=2 -C target-feature=+crt-static --extern libconfarg=libconfarg.rlib --extern libfile=libfile.rlib --extern libstream=libstream.rlib [COMPONENT]/src/main.rs -o [final_name]
```

with above command, you will the final binary of [COMPONENT] in static final form (aka; statically linked) with optimization level 2 and specific libs (crates).

As with cargo, I reccomend do "strip" to the final binaries to delete debug symbols.

### Before - After strip binary

Before strip;
```bash
-rwxr-xr-x 2 shyanjmc shyanjmc 9.0M Oct 23 04:31 ls
```

After strip;
```bash
-rwxr-xr-x 2 shyanjmc shyanjmc 507K Oct 23 04:32 ls
```

The strip command clean the debug symbols, which are the 94.49869791666667% of space.

## Roadmap

### Show

Show binary is used to print file's data and metadata

- [X] File's data (normal).
- [X] File's size (-s).
- [X] File's lines (-l).
- [X] File's owner (-o).
- [X] File's permissions (-p).
- [X] File's datetime modification (-d).
- [X] File's words and letters (-w).
- [X] System's processes (Linux) (--proc).
- [X] Output mode in hexadecimal (--hexa).
- [X] Clean mode (-c).

### Ls

Ls binary is used to print directories content.

- [X] Directories files and sub-directories number (-l).
- [X] Directoyies and files verbose (-v).
- [X] Clean mode (-c).
- [X] System's processes (Linux) (--proc).
- [ ] Fix issue with verbose mode in HOME directory.

### Search

Search binary is used to search into the system.

- [X] Search string inside one or more files.
- [X] Search string in directory's name/path.
- [X] Search string in environment variables.
- [X] Search string in system's processes.
- [ ] Search recursively in path.

### Edit

Edit binary is used to edit file's and system's information.

- [ ] Edit environment variable.
- [ ] Edit file.
- [ ] Edit stdin and/or file based in patterns.

### Huginn 

Hugin, from old norse: thought, is the system init.

The binary name is; init (Linux kernel try to load this file name).

- [ ] Read TOML configuration files.
- [ ] Start process based in configuration file.
- [ ] Network Ethernet (with DHCP and DNS) support.
- [ ] Command line interface (C.L.I.) to manage services/daemons.

### Muninn (pkg)

Munin, from old norse: memory - mind, is the package manager.

- [ ] Read TOML configuration files.
- [ ] Install, uninstall packages with and without root access.
- [ ] Install pre-build package or build from source.

## Contact
If you want contact me, you can do it trough:

Email:

- shyanjmc@proton.me
- shyanjmc@protonmail.com
- joaquincrespo96@gmail.com

Linkedin:

- https://www.linkedin.com/in/joaquin-mcrespo/ 

## Contributions and support

For now I am the main and only dev in this project, maybe in the future I will allow collaborations. 

If you want support this project you can;

Join my patreon;

- https://patreon.com/shyanjmc

Donate me crypto;

- Bitcoin (BTC); 16n6GP4XkZiyoGCZei6uxdpohCkPm7eQ7L
- Ethereum (ETH); 0x27219354cC70dE84e7fae0B71E9e2605026b10B2
- Cosmos (ATOM); cosmos1fmyh8kkdmz4wfhec5k5h97g9syl8e9lpufww8n
- DAI (ERC-20); 0x27219354cC70dE84e7fae0B71E9e2605026b10B2
- Ravencoin (RVN); RRmpKJyu2TTLA94oXCf9PL3u1dmXUAMTd4

Also you can donate trought crypto-coffee.xyz;

- https://crypto-coffee.xyz/donate/shyanjmc

And if you know me personally, let me know that you have donated, since that moment we will share a beer (or mead if you are man/women of honor).
