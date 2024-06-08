# Rune

Rune is the RavnOS' shell.

The shell is the program where you insert commands.

## Limitations

By default the Rust Standard Library (stdlib) is not async because of that;

1. You can not use right and left arrows to edit command without delete first.
2. You can not use up and down arrows to change stdin with older commands.
3. You can not use tab key to complete binary's, directory's or file's name.

As RavnOS design is not use external crates, I must design some replacement for many stdlib's functions to do 
they async and allow those features.

## Architecture

I designed rune to be most bullet proof as is possible, becuase of that;

1. Rune contains the core utilities as builtins.

   A builtin is an internal command executed by you.

   You have two ways to get builtins information;

   > _list

   and/or

   > _help

   The first (_list) will provide you the builtins list, while the second (_help) will provide you with the information about how to use each one.

2. To exit execute; "_exit"

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
