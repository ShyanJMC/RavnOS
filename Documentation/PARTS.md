# RavnOS parts

<!-- TOC START min:1 max:5 link:true asterisk:false update:true -->
- [RavnOS parts](#ravnos-parts)
  - [Core](#core)
  - [Utils](#utils)
  - [Libs](#libs)
<!-- TOC END -->



## Core

- Show

   Used to show information about files, files and system metadata.

- Ls

   Used to show files and directories in specific path with metadata support.

- Search

   Used to search string over files, directories, processes, environment variables and stdin.

- Edit

   Used to edit environment variables, files, directories and stdin.

- Huginn (from old norse; thought )

   Is the sysinit.

- Muninn ( from old norse; memory / mind )

   Is the package manager with support for build from source or install from pre-built binary. Has support for install/build with or without root access.

- Futhark ( the old norse alphabet )

   Is the network manager.

- Rune

   Is the shell.

- Web

   Is the tui web browser and downloader files.

## Utils

- Microhttp

   Is a simple and static web server with support for HTTP/2 (TCP) and HTTP3 (UDP) only. Come with reverse proxy support.

- BlackRavnTunnel

   Is a tunnel for TCP/UDP with encryption support.

## Libs


- libfile

   Crate / lib for files;

   - which(binary: String) -> Vec<String>

      Gets an executable binary name and returns a strings vector where is found.
      Search in PATH env.

   - decode_base64(base64: &String) -> Result<Vec<u8>, String>

      Gets the base64 string and returns the binary (Vec<u8>) or an error.

   - For u64 types;

      - size_to_human(&self) -> String

         Gets the size in bytes and returns in human.

   - For file types;

      - is_binary(&self) -> bool

         Gets the file and returns if is a binary (like an executable) or not.

      - encode_base64(&self) -> String;

         Gets the files and returns it encoded as Base64.

- libstream

   Crate / lib for data streams;

   - binary_search<'a>(filename: &String, mut file: File, ssearch: String) -> Result<(), &'a str>

      Search in a file with name "filename" the string "ssearch" and returns a Result if matchs or not.

   - getprocs() -> Vec<String>

      Returns the processes running in the system. Read the "/proc".

   - file_filter(filename: &String, input: String) -> Vec<String>

      Open "filename" and search the "input" in it, returning the lines that contains it.

   - For Strings types

      - readkey(&self) -> HashMap<String, String>

         Read the stream [key] { [data] }, and returns a HashMap with [key], [data] to extract the data from key.

      - readdir(&self) -> Vec<PathBuf>

         Read the directory and returns elements.

      - readdir_recursive(&self) -> DirStructure

         Read the directory and returns elements recursive (files and directories).

      - word_count(&self) -> Vec<usize>

         Read the string and striping some characters counts them.

      - permission_to_human(&self) -> Vec<&'static str>

         Read the Unix permission type and transform to human type.

   - For i64 types

      - epoch_to_human(&self) -> String

         Read the epoch unix time and transform to UTC-0 / GMT-0 human time.    
