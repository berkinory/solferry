//! Help text longer than a doc comment, kept apart so `args.rs` stays readable.

pub const SHORT: &str = "Scripting, JSON output and exit codes are covered in `solferry --help`.";

pub const AGENTS: &str = r#"Scripts and agents:
  --json        Every command prints JSON to stdout, one value per line.
                  status, devices, nearby, relay show, relay check: one document
                  rename, forget, approval, block, unblock, relay add/remove:
                    one {"action": ...} object
                  pair, join, send, receive, link: {"event": ...} lines
                Device ids are in "id" or "peer". A transfer's "id" is the same
                across its offer, started, progress, completed and failed events.
  Errors        With --json, a failure prints one line to stderr:
                  {"error": "<kind>", "message": "...", "next": "..." or null}
                kind is a stable identifier such as unknown_device, not_paired,
                unreachable, declined, ticket_expired or relay_check_failed.
  Answers       receive without --yes asks before taking files from a device
                whose approval is "ask", and before pairing. With --json it
                prints an "offer" or "pairing_request" event, then reads one
                line from stdin: accept, decline or block. End of input declines.
  --yes         receive takes every file offer without asking, from paired and
                unpaired devices alike. Pairing requests are still asked.
  --once        receive exits after its first finished incoming transfer.
  --data-dir    Where the device key, settings and paired devices live. Also
                SOLFERRY_DATA_DIR. Each directory is a separate device, so two
                directories let two devices run on one machine.
  SOLFERRY_LOG  Log filter for stderr, such as SOLFERRY_LOG=info.
  Exit codes    0 done, 1 failed (the error says why), 2 invalid arguments.

Typical flow:
  A$ solferry pair --json          # {"event":"pair_code","code":"sf1...",...}
  B$ solferry join sf1... --json   # both sides print {"event":"paired",...}
  A$ solferry receive --yes --once --json --dir ./inbox
  B$ solferry send report.pdf --to A --json   # A's name or id prefix
"#;

pub const SEND: &str = r#"Each target is a paired device's name or id prefix, a full device id, or the name
of a device on this network. Unpaired devices are asked before files arrive.
With --json, transfer events stream as lines, then one "sent" or "send_failed"
line per target.

Examples:
  solferry send photo.jpg notes/ --to laptop
  solferry send big.iso --to laptop --to phone --json
  echo hello | solferry send --text - --to laptop"#;

pub const RECEIVE: &str = r#"Offers from devices whose approval is "auto" arrive without asking. Others ask,
unless --yes is given. With --json each question is one line on stdout,
  {"event":"offer","id":...,"peer":...,"peer_name":...,"paired":...,
   "files":...,"bytes":...,"first_path":...}
  {"event":"pairing_request","peer":...,"peer_name":...,"code":...}
answered by one line on stdin: accept, decline or block. Questions come one at a
time and each waits for its answer. End of input declines. The first line is a
"ready" event once the device is listening. Received text does not count for
--once.

Examples:
  solferry receive --dir ~/inbox
  solferry receive --yes --once --json
  echo accept | solferry receive --once --json"#;

pub const LINK: &str = r#"Prints addresses that a browser on the same network can open, then runs until
Ctrl-C. Files taken from a browser go to the download folder that `solferry status`
shows.

Examples:
  solferry link report.pdf slides.key
  solferry link
  solferry link report.pdf --json   # {"event":"link_ready","urls":[...],...}"#;

pub const PAIR: &str = r#"The code works once, for 10 minutes. pair waits until the other device joins, and
exits with ticket_expired if nobody does. With --with or --address, both screens
show a short code to compare, and the other device, running `solferry receive`
or the app, must accept.

Examples:
  solferry pair                   # then `solferry join <code>` on the other device
  solferry pair --json            # {"event":"pair_code",...}, later {"event":"paired",...}
  solferry pair --with laptop     # {"event":"compare_code",...} with --json"#;

pub const JOIN: &str = r#"The other device must still be running `solferry pair`.

Examples:
  solferry join sf1...
  solferry join sf1... --json   # {"event":"paired","peer":...,"peer_name":...}"#;

pub const NEARBY: &str =
    "Listens for a few seconds. Only devices running solferry that are visible to
everyone show up.

Examples:
  solferry nearby
  solferry nearby --json
  solferry nearby --address 192.168.1.20";
