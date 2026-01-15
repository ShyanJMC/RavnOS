# Rune

Rune is the RavnOS' shell.

The shell is the program where you insert commands.

<!-- TOC depthFrom:1 depthTo:6 withLinks:1 updateOnSave:1 orderedList:0 -->

- [Rune](#rune)
	- [Limitations](#limitations)
	- [Architecture](#architecture)
	- [History](#history)
	- [Alias](#alias)
	- [External programs](#external-programs)
	- [Redirections](#redirections)
	- [Scripts](#scripts)
	- [Builtins](#builtins)

<!-- /TOC -->

## Limitations

By default the Rust Standard Library (stdlib) is not async because of that;

1. You can not use right and left arrows to edit command without delete first.
2. You can not use up and down arrows to change stdin with older commands.
3. You can not use tab key to complete binary's, directory's or file's name.
4. In OpenBSD rune can not create automatically the ".ravnos" folder in user's home.

As RavnOS design is not use external crates, I must design and create a async engine to bypass those limitations.

## Architecture

I designed rune to be most bullet proof as is possible, becuase of that.

1. Rune contains the core utilities as builtins.

   A builtin is an internal command executed by you.

   You have two ways to get builtins information;

   > _list

   and/or

   > _help

   The first (_list) will provide you the builtins list, while the second (_help) will provide you with the information about how to use each one.

2. To exit execute; "_exit"

3. Each builtin's options is printed with [builtin] [--help / -h]

## History

Rune enable by default the command history. Is saved by default in;

```
~/.ravnos/rune_history
```

If can not open or append/write in the file will use the temporary directory;

```
/tmp/.ravnos/rune_history
```

Each command will have specified the complete timestamp with this sintax;

```bash
[command_number] [ [day]/[month]/[year] [time] UTC-0] : [command]
```

You can disable with built-in;

> _disable_history

Or enable again with;

> _enable_history

## Alias

Rune support use alias for commands and built-ins. They are specified line by line in;

```
~/.ravnos/rune_alias
```

Each alias is specified using this simple sintax;

```bash
[alias] = [command and arguments]
```

Be careful because the alias can replace everything in the command line.

## External programs

Rune locate the binary to execute searching in directories specified in "PATH" environment variable. So, if can not find the binary to execute,
verify (with _env builtin ) it first.

## Redirections

Rune for now can only use one reditection (>, 2> or pipeline). I'm working to expand it to 3 (three).

> [A] > [B]

> [A] 2> [B]

> [A] | [B]

## Scripts

Rune for now can not execute scripts from files. I will be working on it in the future.

## Builtins

Rune have builtins to replace the basic tools in the system.

Right now these are;

> _base64 [file] [file_n]

	Encode file/s into base64.

> _basename

	Takes a path and prints the last filename.

> _cd [PATH]

	If path do not exist, goes to user home directory.

> _clear

	Clean the screen.

> _count [file]

	Show the file's number lines and words

> _cp [source] [destination]

	Copy file or directory from [source] to [destination].

> _date

	Display the current time and date in UTC-0 (which is the same that GTM-0).

> _decodebase64 [input] [file]

	Decocde input from base64 to file.

> _disable_history

	Disable save commands to history without truncate the file.

> _du [path]

	Show disk usage ('du') in [path], read recusively.

> _enable_history

	Enable save commands to history.

> _echoraw

	Show string into stdout without interpreting special characters.

> _env

	Show environment variables.

> _exit

	Exit the shell properly.

> _expand

	Convert tabs to spaces in file (with new file; [FILE]-edited), with '-t X' you can specify the spaces number, first the options (if exists) and then the file.

> _false [option]

	Returns a false value, '-n' for rune native or '-u' for 1 (false in Unix and GNU).

> _head -n [number] [file]

	Show [number] first lines for file.

> _history

	Show the history commands with date and time.

> _home

	Returns the current user's home directory.

> _id [options]

	Show current user, '-n' for name and '-u' for UUID.

> _info

	Show system's information.

> _join [file_1] [file_n] [destination]

	Joins files into destionation file.

> _mkdir [dest]

	Create directory if it has more subdirectories it will create them recursively.

> _mkfile [file]

	Create empty file.

> _move [source] [destination]

	Move files or directory to new location.

> _nl [file]

	Prints each line with number.

> _list

	List builtins like this.

> _ln [source] [dest]

	Creates a link [dest] to [source].

> _ls [options] [path_1] [path_n]

	Lists files and directories in path.

> _proc

	Show process using /proc directory

> _pwd

	Print the current directory.

> _rm [target]

	Delete the file or directory, if the directory have files inside must use '-r' argument to include them.

> _seq [first]:[last]:[increment]

	Start a secuence from [first] to [last] using [increment] as increment.

> _show [options] [file_1] [file_n]

	Show file's content, file's content in hexadecimal, system information or difference.

> _sleep [seconds]:[nanoseconds]

	Waits X seconds with Y nanoseconds.

> _tail [number] [file]

	Show the last [number] lines of [file].

> _which [binary]

	Show where is located the binary based in PATH environment variable.

> _$?

	Print the latest command exit return, not include builtins
