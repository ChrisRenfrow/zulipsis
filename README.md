# Zulipsis

A silly program that randomly changes your Zulip status at a defined interval. The selection of phrases is configurable and organized into three categories, "start", "pause", and "working", each with a corresponding configurable emoji name. When the program starts, one "start" phrase is selected and sent. Then, at the configured interval, a new "working" phrase is selected and sent. When the program is interrupted (Ctrl+C), one last phrase is selected from the "pause" category and sent.

**Example:**

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
Use this software at your own discretion. At the time of writing I've spent only a few hours writing this, there are likely bugs. This program makes use of your personal Zulip access key, which is effectively a password to your Zulip account to potentially perform destructive operations on your behalf. I encourage you to read the code before running. If you have any questions or concerns please feel free to open a new issue or send me an email.

## Usage

``` txt
Usage: zulipsis [OPTIONS] --zuliprc <ZULIPRC> --config <CONFIG>

Options:
  -z, --zuliprc <ZULIPRC>  The path to zuliprc
  -c, --config <CONFIG>    The path to the config
  -s, --skip <SKIP>        Skip sending the start and/or pause statuses [possible values: start, pause, both]
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

### No default config? What gives?

I haven't added a default configuration yet (sorry). Here's an example you can copy/paste.

``` toml
[general]
cycle_duration_seconds = 300 # 5 minutes

[phrases]
# The initial status to set before cycling over the "working" list
# Sets away to false
start = [
	"Initializing...",
	"Loading...",
	"Network connection established...",
	"Brewing coffee...",
	"Reticulating splines...",
]
# The list of phrases cycled through when online.
# Cycles periodically.
# Sets away to false.
working = [
	"Processing...",
	["Sipping coffee...", "coffee"], # Specify the name of a unique emoji for any phrase like this.
	["Thinking hard...", "brain"],
	"Reading the error...",
	"Reading the manual...",
    "Listening to the compiler...",
]
# The list of phrases to select from as a parting status.
# Does not cycle.
# Sets status as away.
pause = [
	"Suspending...",
	"Hibernating...",
	"Touching grass...",
	"Taking a break...",
]

# The name of the emoji to use by default for each state
[emoji]
start = "start"
working = "tools"
pause = "zzz"
```

## Roadmap

**Absolutely:**

- [x] Add ability to specify emoji on per-phrase basis
  - e.g. `["Rewriting it in Rust...", "ferris"]`
- [ ] Add default configuration
- [ ] Add options to control output (--verbose)
- [ ] Handle common HTTP responses sensibly
- [ ] Make the program behave as expected when running (and terminating) as daemon

**Maybe:**

- [ ] Make behavior more configurable with different modes
- [ ] Make it interoperable with other programs (enables the ability to achieve rich-presence à la Discord)
