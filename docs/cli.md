# Command line

`solferry` does everything the apps do, from a terminal. It suits servers, scripts and machines without a screen. The macOS app and the Windows installer include it, and Linux has its own download. See [Installation](install.md).

Every command accepts `--help` with examples. `solferry --help` lists them all.

## Getting started

```sh
solferry status                         # this device's name, id, folders and relays
solferry rename my-laptop               # the name other devices see
solferry nearby                         # devices on this network and your Tailscale network
```

Pair with a device on the same network by comparing a six digit code. The other device must be running `solferry receive` or the app:

```sh
solferry pair --with my-phone
```

For a device on another network, create a code here and redeem it there:

```sh
solferry pair                           # prints a sf1... code and a QR code
solferry join sf1...                    # on the other device
```

## Sending

```sh
solferry send report.pdf photos/ --to my-laptop
solferry send big.iso --to my-laptop --to my-phone   # several devices at once
solferry send notes.md --to last                     # the device you last sent files to
```

A target is a paired device's name or id prefix, a full device id, or the name of a device on this network. Unpaired devices are asked before anything arrives.

Pipes work too. A lone `-` sends stdin as one file, named with `--name`:

```sh
tar c photos/ | solferry send - --name photos.tar --to my-laptop
```

Send text to a paired device's clipboard with `--text`, or pipe it in:

```sh
solferry send --text "see you at 8" --to my-laptop
git log -1 | solferry send --text - --to my-laptop
```

Ctrl-C stops a send and tells the other side at once.

## Sending later

With `--queue`, a paired device that can't be reached gets the send later instead of failing. It waits for up to 7 days, and the device is still asked first.

```sh
solferry send report.pdf --to my-laptop --queue
solferry queue                          # list waiting sends
solferry queue --cancel <id>            # take one back
```

`solferry receive` and the desktop app deliver waiting sends while they run.

## Receiving

```sh
solferry receive                        # stay online, ask about each offer
solferry receive --dir ~/inbox          # save somewhere else
solferry receive --yes                  # accept paired devices without asking
solferry receive --once                 # exit after the first finished transfer
```

Devices you set to `auto` with `solferry approval` arrive without a question. `--yes` accepts every other paired device too. Unpaired devices are always asked, and on a machine with no terminal to ask on they're refused with one line on stderr.

To keep a server receiving, use the systemd unit in [deploy/systemd/solferry.service](../deploy/systemd/solferry.service).

## Managing devices

```sh
solferry devices                        # paired and blocked devices
solferry approval my-phone auto         # files from my-phone arrive without asking
solferry approval my-phone ask          # ask again
solferry forget my-phone                # unpair
solferry block <device>                 # refuse every connection from it
solferry unblock <id>
```

## Browsers

Share files with any browser on the same network, or take files from one:

```sh
solferry link report.pdf slides.key     # prints addresses a browser can download from
solferry link                           # a page that sends files and text here
```

It runs until Ctrl-C.

## Relays

```sh
solferry relay add 'https://relay.example.com#token'
solferry relay show
solferry relay check                    # tests the server, the token and the UDP port
solferry relay limit 5                  # at most 5 MB/s through relays, 0 or off for none
solferry relay remove https://relay.example.com
```

See [Run a relay](relay.md) for setting one up.

## Scripting

`--json` prints one JSON value per line on stdout, and errors as JSON on stderr. With `receive --json`, offers and pairing requests arrive as events and are answered by one line on stdin: `accept`, `decline` or `block`. End of input declines.

```sh
solferry pair --json                    # {"event":"pair_code","code":"sf1...",...}
solferry join sf1... --json             # both sides print {"event":"paired",...}
solferry receive --yes --once --json --dir ./inbox
solferry send report.pdf --to laptop --json
```

| Option or variable | Effect |
|---|---|
| `--data-dir`, `SOLFERRY_DATA_DIR` | Where the key, settings and paired devices live. Each folder is a separate device, so two folders run two devices on one machine |
| `SOLFERRY_LOG` | Log filter for stderr, such as `SOLFERRY_LOG=info` |

Exit codes: `0` done, `1` failed with an error that says why and what to do, `2` invalid arguments.
