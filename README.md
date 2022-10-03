# RavnOS
RanvOS is a operative system programmed in Rust.

Aims to be; minimal, fast, secure and modern ( this maybe you know as; state-of-art ).

## Target
The target of this project is not do the next Linux, GNU HURD or Redox. Is my personal project to make from scratch an operative system with special focus in minimalism, portability, cyber security and wherever I need in the future.

Take this project as a hobby, nothing professional.

I prefer build my programs as statically linked, with native CPU support and without debug information. Because of that I recommend set this configuration in " ~/.cargo/config.toml ":

> \[build]
> 
> jobs = 20
> 
> 
> rustc = "rustc"
> 
> rustflags = ["-C","opt-level=2","-C","debuginfo=0","-C", "target-feature=+crt-static","-C","target-cpu=native"]

## Standard

When I writte RavnOS I just put this topics as standard for development, distribution and use;

1. No external crates allowed.

2. The musl lib is used for compile.

3. All must be compiled as static binary. Becuase of this musl is the best option.

## Note
I like do this project thiking; 

> if you are in the middle of mountain without Starlink or any other internet connection, how you will build this?

Because of that, and one fun challenge, is not add dependencies that need internet, all must be local. 

## Requirements 
- Rustc
- A terminal
- Cargo with the target toolchain you want/need.
- Musl target: This is not mandatory, but I recommend it to build as static.

## Build

```rust
cargo build --release --target [x86_64/arm64/etc]-unknown-[linux/windows/etc]-musl
```

All binaries will be in "target/\[TARGET]/release".

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
