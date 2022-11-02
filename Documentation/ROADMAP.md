# Roadmap

## 0.1.0 ALPHA

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
- [ ] Recognition of special characters in stdin mode (like; \n , \t and others) with EOF as delimiter.
- [ ] Difference between two files.

### Ls


- [X] Directories files and sub-directories number (-l).
- [X] Directoyies and files verbose (-v).
- [X] Clean mode (-c).
- [X] System's processes (Linux) (--proc).
- [X] Fix issue with verbose mode in HOME directory. ---> Was an issue in my system with Steam folder.

### Search

- [X] Search string inside one or more files.
- [X] Search string in directory's name/path.
- [X] Search string in environment variables.
- [X] Search string in system's processes.
- [ ] Search recursively in path.
- [ ] Search in stdin.

### Edit

- [ ] Edit environment variable.
- [ ] Edit file.
- [ ] Edit stdin and/or file based in patterns.

### Huginn 

- [ ] Read TOML configuration files.
- [ ] Start process based in configuration file.
- [ ] Socket support.
- [ ] Command line interface (C.L.I.) to manage services/daemons.

### Muninn (pkg)

- [ ] Read TOML configuration files.
- [ ] Install, uninstall packages with and without root access.
- [ ] Install pre-build binary or build from source.

### Rune 

- [ ] Read TOML configuration file.
- [ ] Support for pipelines ("|") and redirections (<, >).
- [ ] Support for create files and directories.

### Futhark
- [ ] Ethernet support.
- [ ] DNS and DNSSEC support.
- [ ] TCP/IP and UDP support.