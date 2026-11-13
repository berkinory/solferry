#!/usr/bin/env python3
"""Runs solferry through NAT, loss, outages and restarts. Needs passwordless sudo, Linux only."""

import hashlib
import json
import re
import sys
import tempfile
import threading
import time
from pathlib import Path

import lab

REPO = Path(__file__).resolve().parents[2]
SOLFERRY = REPO / "target/release/solferry"
RELAY = sys.argv[1] if len(sys.argv) > 1 else "iroh-relay"


def digest(path):
    with open(path, "rb") as file:
        return hashlib.file_digest(file, "blake2b").hexdigest()


class Stage:
    def __init__(self, nat="cone", size_mib=200):
        self.work = Path(tempfile.mkdtemp(prefix="solferry-netlab-"))
        self.work.chmod(0o755)
        lab.up(nat)
        lab.start_relays(RELAY, self.work)
        lab.configure(self.work / "a", "alpha", [lab.RELAY_X])
        lab.configure(self.work / "b", "beta", [lab.RELAY_Y])
        self.a = lab.Host("solferry-a", self.work / "a", SOLFERRY)
        self.b = lab.Host("solferry-b", self.work / "b", SOLFERRY)
        rc, output, _ = lab.pair(self.a, self.b, self.work)
        if rc:
            raise RuntimeError(f"pairing failed: {output}")
        self.file = self.work / "payload.bin"
        with open("/dev/urandom", "rb") as source, open(self.file, "wb") as target:
            for _ in range(size_mib):
                target.write(source.read(1024 * 1024))
        self.inbox = self.work / "inbox"
        self.receiver = None

    def listen(self):
        self.receiver = self.b.spawn(["receive", "--yes", "--dir", self.inbox], self.work / "receive.log")
        time.sleep(1.5)

    def send(self, timeout=240):
        return self.a.run(["send", self.file, "--to", "beta"], timeout=timeout)

    def delivered(self):
        target = self.inbox / self.file.name
        return target.exists() and digest(target) == digest(self.file)

    def close(self):
        self.a.stop()
        self.b.stop()


def speed(seconds, mib):
    return f"{mib / seconds:.0f} MiB/s" if seconds else "?"


def summary(rc, output, seconds, ok, mib):
    route = re.search(r"over (.+?)(,|$)", output, re.M)
    resumed = re.search(r"resumed ([\d.]+ \w+)", output)
    return {"rc": rc, "delivered": ok, "route": route and route.group(1), "seconds": round(seconds, 1), "speed": speed(seconds, mib), "resumed": resumed and resumed.group(1), "tail": output.splitlines()[-2:]}


def in_background(seconds, action):
    timer = threading.Timer(seconds, action)
    timer.start()
    return timer


def nat_cone():
    stage = Stage("cone")
    stage.listen()
    rc, output, seconds = stage.send()
    return summary(rc, output, seconds, stage.delivered(), 200), stage


def nat_symmetric_own_relay():
    stage = Stage("symmetric")
    stage.listen()
    rc, output, seconds = stage.send()
    return summary(rc, output, seconds, stage.delivered(), 200), stage


def nat_no_relay():
    work = Path(tempfile.mkdtemp(prefix="solferry-netlab-"))
    work.chmod(0o755)
    lab.up("cone")
    lab.configure(work / "a", "alpha", [])
    lab.configure(work / "b", "beta", [])
    inviter, joiner = lab.Host("solferry-a", work / "a", SOLFERRY), lab.Host("solferry-b", work / "b", SOLFERRY)
    rc, output, seconds = lab.pair(inviter, joiner, work)
    stage = type("Unreachable", (), {"close": lambda self: (inviter.stop(), joiner.stop())})()
    return {"rc": rc, "seconds": round(seconds, 1), "tail": output.splitlines()[-2:]}, stage


def relay_from_pairing():
    work = Path(tempfile.mkdtemp(prefix="solferry-netlab-"))
    work.chmod(0o755)
    lab.up("symmetric")
    lab.start_relays(RELAY, work)
    lab.configure(work / "a", "alpha", [lab.RELAY_X])
    lab.configure(work / "b", "beta", [])
    (work / "b" / "config.json").write_text(json.dumps({"name": "beta", "relay_ca_file": str(work / "ca.pem")}))
    sender, receiver = lab.Host("solferry-a", work / "a", SOLFERRY), lab.Host("solferry-b", work / "b", SOLFERRY)
    rc, output, _ = lab.pair(sender, receiver, work)
    if rc:
        raise RuntimeError(f"pairing failed: {output}")
    payload = work / "payload.bin"
    with open("/dev/urandom", "rb") as source, open(payload, "wb") as target:
        target.write(source.read(20 * 1024 * 1024))
    receiver.spawn(["receive", "--yes", "--dir", work / "inbox"], work / "receive.log")
    time.sleep(3)
    rc, output, seconds = sender.run(["send", payload, "--to", "beta"])
    target = work / "inbox/payload.bin"
    ok = target.exists() and digest(target) == digest(payload)
    stage = type("Shared", (), {"close": lambda self: (sender.stop(), receiver.stop())})()
    return summary(rc, output, seconds, ok, 20), stage


def relay_wrong_token():
    work = Path(tempfile.mkdtemp(prefix="solferry-netlab-"))
    work.chmod(0o755)
    lab.up("symmetric")
    lab.start_relays(RELAY, work)
    lab.configure(work / "a", "alpha", [lab.RELAY_X.replace(lab.RELAY_TOKEN, "wrong")])
    lab.configure(work / "b", "beta", [])
    (work / "b" / "config.json").write_text(json.dumps({"name": "beta", "relay_ca_file": str(work / "ca.pem")}))
    inviter, joiner = lab.Host("solferry-a", work / "a", SOLFERRY), lab.Host("solferry-b", work / "b", SOLFERRY)
    rc, output, seconds = lab.pair(inviter, joiner, work)
    stage = type("Refused", (), {"close": lambda self: (inviter.stop(), joiner.stop())})()
    return {"rc": rc, "seconds": round(seconds, 1), "tail": output.splitlines()[-2:]}, stage


def relay_rotation():
    """Relay x gets a new token. Beta only knows the old one, from pairing, and must keep working
    through the grace period and after the old token stops."""
    work = Path(tempfile.mkdtemp(prefix="solferry-netlab-"))
    work.chmod(0o755)
    lab.up("symmetric")
    lab.start_relays(RELAY, work)
    lab.configure(work / "a", "alpha", [lab.RELAY_X])
    (work / "b").mkdir()
    (work / "b" / "config.json").write_text(json.dumps({"name": "beta", "relay_ca_file": str(work / "ca.pem")}))
    sender, receiver = lab.Host("solferry-a", work / "a", SOLFERRY), lab.Host("solferry-b", work / "b", SOLFERRY)
    stage = type("Rotated", (), {"close": lambda self: (sender.stop(), receiver.stop())})()
    rc, output, _ = lab.pair(sender, receiver, work)
    if rc:
        raise RuntimeError(f"pairing failed: {output}")
    payloads = [work / "first.bin", work / "second.bin"]
    for payload in payloads:
        with open("/dev/urandom", "rb") as source, open(payload, "wb") as target:
            target.write(source.read(20 * 1024 * 1024))
    rotated = "netlab-rotated"
    lab.restart_relay_x(RELAY, work, [rotated, lab.RELAY_TOKEN])
    sender.run(["relay", "add", lab.RELAY_X.replace(lab.RELAY_TOKEN, rotated)])
    receiver.spawn(["receive", "--yes", "--dir", work / "inbox"], work / "receive.log")
    time.sleep(3)

    def deliver(payload):
        rc, output, seconds = sender.run(["send", payload, "--to", "beta"])
        target = work / "inbox" / payload.name
        return summary(rc, output, seconds, target.exists() and digest(target) == digest(payload), 20)

    grace = deliver(payloads[0])
    time.sleep(2)
    stored = json.loads((work / "b" / "devices.json").read_text())[0]["relays"]
    adopted = any(link.endswith("#" + rotated) for link in stored)
    lab.restart_relay_x(RELAY, work, [rotated])
    time.sleep(5)
    finished = deliver(payloads[1])
    return {"grace": grace, "beta_adopted_new_token": adopted, "after_finish": finished}, stage


def relay_speed_limit():
    """A 5 MB/s relay speed limit on the sender, then on the receiver, then on both over a direct
    path, which must not be limited."""
    results = {}
    for label, nat, limited in [
        ("relay_send_capped", "symmetric", "a"),
        ("relay_receive_capped", "symmetric", "b"),
        ("direct_uncapped", "cone", "ab"),
    ]:
        stage = Stage(nat, 40)
        for host in limited:
            getattr(stage, host).run(["relay", "limit", "5"])
        stage.listen()
        rc, output, seconds = stage.send()
        results[label] = summary(rc, output, seconds, stage.delivered(), 40)
        stage.close()
    return results, type("Done", (), {"close": lambda self: None})()


def bad_link(netem):
    def run():
        stage = Stage("cone", 300)
        for router in ["solferry-rx", "solferry-ry"]:
            lab.ns(router, f"tc qdisc add dev wan0 root netem {netem}")
        stage.listen()
        rc, output, seconds = stage.send(200)
        return summary(rc, output, seconds, stage.delivered(), 300), stage

    return run


def outage(seconds_down):
    def run():
        stage = Stage("cone", 1500)
        stage.listen()
        lab.ns("solferry-rx", "tc qdisc add dev wan0 root netem rate 100mbit")
        block = lambda: lab.ns("solferry-rx", "iptables -I FORWARD -j DROP")
        unblock = lambda: lab.ns("solferry-rx", "iptables -D FORWARD -j DROP")
        in_background(4, block)
        in_background(4 + seconds_down, unblock)
        rc, output, seconds = stage.send(400)
        return summary(rc, output, seconds, stage.delivered(), 1500), stage

    return run


def receiver_restart():
    stage = Stage("cone", 1500)
    lab.ns("solferry-rx", "tc qdisc add dev wan0 root netem rate 200mbit")
    stage.listen()

    def restart():
        stage.b.stop()
        time.sleep(3)
        stage.listen()

    in_background(14, restart)
    rc, output, seconds = stage.send(400)
    return summary(rc, output, seconds, stage.delivered(), 1500), stage


def address_change():
    stage = Stage("cone", 1500)
    lab.ns("solferry-rx", "tc qdisc add dev wan0 root netem rate 200mbit")
    stage.listen()

    def move():
        lab.ns("solferry-a", "ip addr add 10.1.0.77/24 dev eth0")
        lab.ns("solferry-a", "ip addr del 10.1.0.2/24 dev eth0")
        lab.ns("solferry-a", "ip route replace default via 10.1.0.1")

    in_background(10, move)
    rc, output, seconds = stage.send(400)
    return summary(rc, output, seconds, stage.delivered(), 1500), stage


def direct_path():
    stage = Stage("cone", 2000)
    stage.listen()
    rc, output, seconds = stage.send()
    return summary(rc, output, seconds, stage.delivered(), 2000), stage


def offline_lan():
    work = Path(tempfile.mkdtemp(prefix="solferry-netlab-"))
    work.chmod(0o755)
    lab.lan()
    lab.configure(work / "a", "alpha", [])
    lab.configure(work / "b", "beta", [])
    sender, receiver = lab.Host("solferry-a", work / "a", SOLFERRY), lab.Host("solferry-b", work / "b", SOLFERRY)
    payload = work / "payload.bin"
    with open("/dev/urandom", "rb") as source, open(payload, "wb") as target:
        target.write(source.read(500 * 1024 * 1024))
    receiver.spawn(["receive", "--yes", "--dir", work / "inbox"], work / "receive.log")
    time.sleep(3)
    rc, output, seconds = sender.run(["send", payload, "--to", "beta"])
    target = work / "inbox/payload.bin"
    ok = target.exists() and digest(target) == digest(payload)
    stage = type("Offline", (), {"close": lambda self: (sender.stop(), receiver.stop())})()
    return summary(rc, output, seconds, ok, 500), stage


SCENARIOS = {
    "nat-cone": nat_cone,
    "nat-symmetric-own-relay": nat_symmetric_own_relay,
    "nat-no-relay": nat_no_relay,
    "relay-from-pairing": relay_from_pairing,
    "relay-wrong-token": relay_wrong_token,
    "relay-rotation": relay_rotation,
    "relay-speed-limit": relay_speed_limit,
    "direct-throughput": direct_path,
    "offline-lan": offline_lan,
    "wifi-link": bad_link("delay 5ms loss 0.5% rate 300mbit"),
    "lossy-link": bad_link("delay 40ms loss 1% rate 200mbit"),
    "jitter-link": bad_link("delay 40ms 10ms loss 1% rate 200mbit"),
    "far-link": bad_link("delay 50ms rate 1gbit limit 100000"),
    "outage-10s": outage(10),
    "outage-45s": outage(45),
    "receiver-restart": receiver_restart,
    "address-change": address_change,
}


def main():
    wanted = sys.argv[2:] or list(SCENARIOS)
    results = {}
    for name in wanted:
        print(f"== {name}", flush=True)
        try:
            results[name], stage = SCENARIOS[name]()
            stage.close()
        except Exception as error:
            results[name] = {"error": repr(error)[:300]}
        print(json.dumps(results[name], ensure_ascii=False), flush=True)
        for work in Path(tempfile.gettempdir()).glob("solferry-netlab-*"):
            lab.sh(f"rm -rf {work}")
    lab.down()


if __name__ == "__main__":
    main()
