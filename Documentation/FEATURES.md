# Features

At today, the latest version is; v0.31.0-ALPHA

## Show

- [X] File's data (normal).
- [X] File's size (-s).
- [X] File's lines (-l).
- [X] File's owner (-o).
- [X] File's permissions (-p).
- [X] File's datetime modification (-d).
- [X] File's words and letters (-w).
- [X] System's processes (Linux) (-\-proc).
- [X] Output mode in hexadecimal (--hexa).
- [X] Clean mode (-c).
- [X] Stdin mode (-\-stdin).
- [X] Recognition of environment variables (-e).
- [X] Recognition of special characters in stdin mode (like; \n , \t and others) with EOF (this depends of shell, file format, etc) as delimiter.
- [X] Difference between two files. (-\-diff)
- [X] Current date time (-\-date).
- [X] Show system information (like distro, kernel version, etc) (-\-info)
   - For now is compatible with Linux kernel, in the future I will add BSD too.
- [X] Adapt to work with; [key] { [data] } syntax.
- [ ] Add support to read binary information and print the same information that; ldd and file.
- [ ] Add support to show kernel's syscalls from a process. Like "strace".
- [ ] Add support to read DMI BIOS table to show information about hardware vendor, hardware model and firmware version.
- [ ] Add support to render image with ASCII / UTF-8/16
- [X] Add support to show absolute location as; "which" (-\-which).
- [X] Add support to show information in BASE64 (-\-base64).
- [X] Add support to save information from BASE64 to binary in file (-\-from-base64)
- [X] Add support to identify if file is binary or not and show properly.
- [ ] Show system's resource like nmon or htop
- [ ] Add support to get specific line number of file

## Microhttp

- [ ] Add support for TCP
- [ ] Add support for UDP
- [ ] Add support for HTTP/2
- [ ] Add support for HTTP/3
- [ ] Add support for read configuration file
- [ ] Add support for security features and hardening
- [ ] Add support for TLSv1.3
- [ ] Add support for reverse proxy

## Web

- [ ] Add support for user-agent and behavior as curl
- [ ] Check use of TLS and security headers
- [ ] Download resource from Internet.
- [ ] Browse webpages without javascript, only HTML and CSS from terminal.


## Search

- [X] Search string inside one or more files.
- [X] Search string in directory's name/path.
- [X] Search string in environment variables.
- [X] Search string in system's processes.
- [X] Search recursively in path.
- [X] Search in stdin.
- [X] Extract data from keys
- [X] Search inside binaries

## Ls

- [X] Directories files and sub-directories number (-l).
- [X] Directoyies and files verbose (-v).
- [X] Clean mode (-c).
- [X] System's processes (Linux) (--proc).
- [X] Fix issue with verbose mode in HOME directory. ---> Was an issue in my system with Steam folder.
- [X] Show modified time in UTC and not in Unix Epoch

## Edit

- [ ] Edit environment variable.
- [ ] Edit file.
- [ ] Edit stdin and/or file based in patterns.
- [ ] Edit file/directory permissions and owner/group.
- [ ] Edit hostname.

## Huginn (sysinit)

- [ ] Read TOML configuration files.
- [ ] Start process based in configuration file.
- [ ] Socket support.
- [ ] Command line interface (C.L.I.) to manage services/daemons.

## Muninn (pkg)

- [ ] Read configuration files.
- [ ] Install, uninstall packages with and without root access.
- [ ] Install pre-build binary or build from source.
- [ ] Support for read PKGBUILD and APKBUILD files.

## Rune (shell)

- [ ] Integrate as builtins all coreutils binaries.
- [ ] Read configuration file.
- [X] Support for builtins called with "_X". Use "_list" to list them.
- [X] Support for read/show the last signal returned by command ($?).
- [X] Support for history (_history).
- [ ] Support for pipelines ("|") and redirections (<, >).
- [X] Support for create files and directories.
- [X] Support for delete files and directories.
- [ ] Support for show information from Cargo.toml file inside directory.
- [ ] Support for read git information.
- [ ] Support to save some process's stdout into image (PNG and JPEG).

## Futhark (net)

- [ ] Ethernet support.
- [ ] DNS and DNSSEC support.
- [ ] TCP/IP (v4) and UDP support.
- [ ] TCP/IP (v6) support.

## BlackRavnTunnel

- [ ] Add support for openvpn protocol.
- [ ] Add support for wireguard protocol.
- [ ] Add support for IKEv2 and IPSec protocols.

---
---
## libs / crates

- [X] Adapt them to work with; [key] { [data] } syntax.
- [ ] Capability to read toml from file.

## libtui

- [ ] Support for charts, bars, stderr, stdout, stdin, sections / frames and keys.
