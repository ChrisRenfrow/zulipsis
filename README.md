# Zulipsis

A program that randomly selects a new Zulip status from a pool of phrases at a defined interval. The selection of phrases is configurable and organized into three categories, "start", "pause", and "working", each with a corresponding configurable emoji name. When the program starts, one "start" phrase is selected and sent. Then, at the configured interval, a new "working" phrase is selected and sent. When the program is interrupted (Ctrl+C), one last phrase is selected from the "pause" category and sent.

**Example Output:**

``` txt
[2023-10-19T19:32:30Z INFO  zulipsis] Sending working status: Sipping coffee...
[2023-10-19T19:37:31Z INFO  zulipsis] Sending working status: Learning generously...
[2023-10-19T19:42:32Z INFO  zulipsis] Sending working status: Re-reading the manual...
```
Later...
``` txt
[2023-10-19T19:52:34Z INFO  zulipsis] Sending working status: Re-reading the error...
^C
[2023-10-19T19:56:17Z INFO  zulipsis] Interrupt received. Sending pause status: Taking a break...
```

## Disclaimer

Use this software at your own discretion. There are likely bugs. This program makes use of your personal Zulip access key, which is effectively a password to your Zulip account that allows potentially destructive operations on your behalf. I encourage you to read the code before running. If you have any questions or concerns please feel free to open a new issue or send me an email explaining the issue.

## Usage

``` txt
Usage: zulipsis [OPTIONS]

Options:
  -z, --zuliprc <ZULIPRC>  The path to zuliprc
  -c, --config <CONFIG>    The path to the config
  -s, --skip <SKIP>        Skip sending the start and/or pause statuses [possible values: start, pause, both]
      --default-config     Print default config (e.g. to redirect to `~/.config/zulipsis/config.toml`)
  -v, --verbose...         More output per occurrence
  -q, --quiet...           Less output per occurrence
  -h, --help               Print help
```

### Where do I find my zuliprc?

1. Visit `[subdomain].zulipchat.com/#settings/account-and-privacy` 
2. Click "Show/change your API key"
3. Click "Download .zuliprc"
4. Put it somewhere safe!

**Note:** You may have to enclose the values in quotation marks as mine were not enclosed and that's not valid TOML according to the parser I'm using.

### Where can I find a default config?
Use the `--default-config` argument to print out a basic default to, for example, redirect to a file located at the default config path, like this: `zulipsis --default-config > ~/.config/zulipsis/config.toml`

Here it is, annotated to explain the purpose of each section:

``` toml
[general]
# How frequently the status gets cycled in seconds
cycle_duration_seconds = 300

[phrases]
# The phrases selected when you start zulipsis
start = [
  # You may supply a basic text phrase like this
  "getting started",
  # Or provide the phrase and the name of an emoji like this to override the default (see below)
  ["waking-up", "sunrise"],
  ["catching-up on zulip", "zulip"]
]
# The phrases cycled through as zulipsis continues to run
working = [
  "working", 
  ["thinking", "brain"], 
  "reading the docs"
]
# The phrases selected when the program receives an interrupt (Ctrl+C)
pause = [
  "taking a break", 
  ["afk", "keyboard"]
]

# The names of the emoji to use by default during each phase
[emoji]
start = "start"
working = "tools"
pause = "zzz"
```

## Roadmap

**Absolutely:**

- [x] Add ability to specify emoji on per-phrase basis
  - e.g. `["Rewriting it in Rust...", "ferris"]`
- [x] Add options to control output (--verbose)
- [x] Add default configuration search paths (making --config and --zuliprc optional)
- [x] Add argument to generate default configuration
- [ ] Optionally retry when encountering (potentially) temporary network interference
- ~~[ ] Respond to SIGKILL like it responds to SIGINT~~ Abandoned, SIGKILL cannot/should-not be subscribed to

**Maybe:**

- [ ] Make behavior more configurable with different modes
- [ ] Make it interoperable with other programs (enables the ability to achieve rich-presence à la Discord)
