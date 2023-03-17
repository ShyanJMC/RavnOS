# Roadmap

<!-- TOC START min:1 max:5 link:true asterisk:false update:true -->
- [Roadmap](#roadmap)
  - [v1.0.0](#v100)
  - [v1.0.0 BETA](#v100-beta)
  - [0.XX.0 ALPHA](#0xx0-alpha)
    - [BlackRavnTunnel](#blackravntunnel)
  - [0.73.0 ALPHA](#0730-alpha)
    - [Show](#show)
    - [Microhttp](#microhttp)
  - [0.70.0 ALPHA](#0700-alpha)
    - [Show](#show-1)
    - [Microhttp](#microhttp-1)
    - [Web](#web)
  - [0.58.0 ALPHA](#0580-alpha)
    - [Search](#search)
  - [0.57.0 ALPHA](#0570-alpha)
    - [Show](#show-2)
    - [Rune (shell)](#rune-shell)
    - [Futhark (net)](#futhark-net)
    - [Web](#web-1)
  - [0.51.0 ALPHA](#0510-alpha)
    - [Futhark (net)](#futhark-net-1)
  - [0.48.0 ALPHA](#0480-alpha)
    - [Rune (shell)](#rune-shell-1)
  - [0.45.0 ALPHA](#0450-alpha)
    - [Muninn (pkg)](#muninn-pkg)
  - [0.42.0 ALPHA](#0420-alpha)
    - [Huginn (sysinit)](#huginn-sysinit)
  - [0.38.0 ALPHA](#0380-alpha)
    - [Edit](#edit)
    - [Show](#show-3)
    - [libtui](#libtui)
  - [0.31.0 ALPHA](#0310-alpha)
    - [Show](#show-4)
    - [Search](#search-1)
    - [libs / crates](#libs--crates)
  - [0.12.0 ALPHA](#0120-alpha)
    - [Search](#search-2)
  - [0.6.0 ALPHA](#060-alpha)
    - [Ls](#ls)
<!-- TOC END -->



## v1.0.0

After some months of beta, depending the amount of bugs detected in v1.0.0-BETA, will be released the first stable release.

---

## v1.0.0 BETA

In this stage will not be added any new feature, just fix bugs and performance improvements.

---

## 0.XX.0 ALPHA

### BlackRavnTunnel

- [ ] Add support for openvpn protocol.
- [ ] Add support for wireguard protocol.
- [ ] Add support for IKEv2 and IPSec protocols.

---

## 0.73.0 ALPHA

### Show

- [ ] Add support to read binary information and print the same information that; ldd and file.
- [ ] Add support to show information in Hexadecimal.

### Microhttp

- [ ] Add support for reverse proxy

---

## 0.70.0 ALPHA

### Show

- [ ] Add support to show kernel's syscalls from a process. Like "strace".
- [ ] Add support to read DMI BIOS table to show information about hardware vendor, hardware model and firmware version.
- [ ] Add support to render image with ASCII / UTF-8/16

### Microhttp

- [ ] Add support for TCP
- [ ] Add support for UDP
- [ ] Add support for HTTP/2
- [ ] Add support for HTTP/3
- [ ] Add support for read configuration file
- [ ] Add support for security features and hardening
- [ ] Add support for TLSv1.3

### Web

- [ ] Add support for user-agent and behavior as curl
- [ ] Check use of TLS and security headers

---

## 0.58.0 ALPHA

### Search

- [ ] Search inside binaries

---

## 0.57.0 ALPHA

### Show

- [ ] Add support to show information in BASE64.

### Rune (shell)
- [ ] Support for read git information.
- [ ] Support to save some process's stdout into image (PNG and JPEG).

### Futhark (net)
- [ ] TCP/IP (v6) support.

### Web
- [ ] Download resource from Internet.
- [ ] Browse webpages without javascript, only HTML and CSS from terminal.

---

## 0.51.0 ALPHA

### Futhark (net)
- [ ] Ethernet support.
- [ ] DNS and DNSSEC support.
- [ ] TCP/IP (v4) and UDP support.

---

## 0.48.0 ALPHA

### Rune (shell)

- [ ] Read configuration file.
- [ ] Support for pipelines ("|") and redirections (<, >).
- [ ] Support for create files and directories (show information from Cargo.toml file inside directory)

---

## 0.45.0 ALPHA

### Muninn (pkg)

- [ ] Read configuration files.
- [ ] Install, uninstall packages with and without root access.
- [ ] Install pre-build binary or build from source.

----

## 0.42.0 ALPHA

### Huginn (sysinit)

- [ ] Read TOML configuration files.
- [ ] Start process based in configuration file.
- [ ] Socket support.
- [ ] Command line interface (C.L.I.) to manage services/daemons.

---

## 0.38.0 ALPHA


### Edit

- [ ] Edit environment variable.
- [ ] Edit file.
- [ ] Edit stdin and/or file based in patterns.
- [ ] Edit file/directory permissions and owner/group.
- [ ] Edit hostname.


### Show
- [ ] Show system's resource like nmon or htop

### libtui

- [ ] Support for charts, bars, stderr, stdout, stdin, sections / frames and keys.


---

## 0.31.0 ALPHA

commit d5de0ec1a5d60858bc74541dcc1ef57b9c1c52fe (HEAD -> master)
Author: ShyanJMC <shyanjmc@protonmail.com>
Date:   Tue Jan 24 04:52:07 2023 -0300

### Show

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


### Search

- [X] Extract data from keys


### libs / crates

- [X] Adapt them to work with; [key] { [data] } syntax.

---

## 0.12.0 ALPHA

- Mon Nov 14 23:45:31 2022 -0300
- commit 15925d1bde7222a22930b85522c29dea0e8f6f0d

### Search

- [X] Search string inside one or more files.
- [X] Search string in directory's name/path.
- [X] Search string in environment variables.
- [X] Search string in system's processes.
- [X] Search recursively in path.
- [X] Search in stdin.


---

## 0.6.0 ALPHA

- Sun Nov 13 19:53:45 2022 -0300
- commit eb499d29c3cd67f5074841cb317d729271c3ca59

### Ls


- [X] Directories files and sub-directories number (-l).
- [X] Directoyies and files verbose (-v).
- [X] Clean mode (-c).
- [X] System's processes (Linux) (--proc).
- [X] Fix issue with verbose mode in HOME directory. ---> Was an issue in my system with Steam folder.
- [X] Show modified time in UTC and not in Unix Epoch
