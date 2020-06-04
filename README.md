# K's bash prompt

Generate PS1 and PS2 prompts for my bash.

```
$ kbashprompt
// outputs $PS1

$ kbashprompt 2
// outputs $PS2
```

## Build

```
make
make install
```

This will put the binary into `/usr/local/bin/`, which can be changed with,
e.g. `PREFIX=/opt make install` to put it in `/opt/bin`.


## Include in .bash_profile

```
# Make sure to keep the single quotes, so that the promp is re-evaluated on every prompt
# Otherwise it would only change if you open a new bash session 
export PS1='$(kbashprompt)';

# PS2 is not dynamic, can be calculated once and reused
export PS2=$(kbashprompt 2);
```

## Dependencies

- Requires a terminal with 256 color support (sends `^[[38;5;{ccc}m` ANSI color codes)
