"""Network namespaces that model two homes behind NAT, joined by a fake internet."""

import json
import os
import shlex
import subprocess
import time
from pathlib import Path

RELAY_TOKEN = "netlab-token"
RELAY_X = f"https://203.0.113.1:3340/#{RELAY_TOKEN}"
RELAY_Y = f"https://203.0.113.2:3340/#{RELAY_TOKEN}"
NAMESPACES = ["beam-wan", "beam-rx", "beam-ry", "beam-a", "beam-b"]
USER = os.environ.get("SUDO_USER") or os.environ["USER"]


def sh(command, check=True):
    return subprocess.run(["sudo", "-n", "sh", "-c", command], check=check, capture_output=True, text=True)


def ns(name, command, check=True):
    return sh(f"ip netns exec {name} {command}", check)


def down():
    for name in NAMESPACES:
        sh(f"ip netns pids {name} | xargs -r kill -9", check=False)
        sh(f"ip netns del {name}", check=False)


def link(left, left_if, right, right_if):
    sh(f"ip link add {left_if} netns {left} type veth peer name {right_if} netns {right}")


def up(mode="cone"):
    down()
    for name in NAMESPACES:
        sh(f"ip netns add {name}")
        ns(name, "ip link set lo up")
    link("beam-rx", "wan0", "beam-wan", "to-rx")
    link("beam-ry", "wan0", "beam-wan", "to-ry")
    link("beam-a", "eth0", "beam-rx", "lan0")
    link("beam-b", "eth0", "beam-ry", "lan0")
    ns("beam-wan", "ip link add br0 type bridge")
    ns("beam-wan", "ip addr add 203.0.113.1/24 dev br0")
    for device in ["br0", "to-rx", "to-ry"]:
        ns("beam-wan", f"ip link set {device} up")
    for device in ["to-rx", "to-ry"]:
        ns("beam-wan", f"ip link set {device} master br0")
    for router, wan_ip, lan, host in [
        ("beam-rx", "203.0.113.10", "10.1.0", "beam-a"),
        ("beam-ry", "203.0.113.20", "10.2.0", "beam-b"),
    ]:
        ns(router, f"ip addr add {wan_ip}/24 dev wan0")
        ns(router, f"ip addr add {lan}.1/24 dev lan0")
        ns(router, "ip link set wan0 up")
        ns(router, "ip link set lan0 up")
        ns(router, "sysctl -qw net.ipv4.ip_forward=1")
        suffix = " --random-fully" if mode == "symmetric" else ""
        ns(router, f"iptables -t nat -A POSTROUTING -o wan0 -j MASQUERADE{suffix}")
        ns(router, "iptables -A INPUT -i wan0 -m conntrack --ctstate NEW -j DROP")
        ns(host, f"ip addr add {lan}.2/24 dev eth0")
        ns(host, "ip link set eth0 up")
        ns(host, f"ip route add default via {lan}.1")


def make_ca(workdir):
    subprocess.run(
        ["openssl", "req", "-x509", "-newkey", "ec", "-pkeyopt", "ec_paramgen_curve:prime256v1", "-nodes", "-days", "2",
         "-keyout", workdir / "ca.key", "-out", workdir / "ca.pem", "-subj", "/CN=beam netlab",
         "-addext", "basicConstraints=critical,CA:TRUE"],
        check=True, capture_output=True,
    )
    subprocess.run(
        ["openssl", "req", "-newkey", "ec", "-pkeyopt", "ec_paramgen_curve:prime256v1", "-nodes",
         "-keyout", workdir / "relay.key", "-out", workdir / "relay.csr", "-subj", "/CN=relay"],
        check=True, capture_output=True,
    )
    extensions = workdir / "san.ext"
    extensions.write_text("subjectAltName=IP:203.0.113.1,IP:203.0.113.2\n")
    subprocess.run(
        ["openssl", "x509", "-req", "-in", workdir / "relay.csr", "-CA", workdir / "ca.pem", "-CAkey", workdir / "ca.key",
         "-CAcreateserial", "-days", "2", "-extfile", extensions, "-out", workdir / "relay.pem"],
        check=True, capture_output=True,
    )


def lan():
    """Two devices on one link with no route anywhere else."""
    down()
    for name in ["beam-a", "beam-b"]:
        sh(f"ip netns add {name}")
        ns(name, "ip link set lo up")
    link("beam-a", "eth0", "beam-b", "eth0")
    for name, address in [("beam-a", "192.168.50.2"), ("beam-b", "192.168.50.3")]:
        ns(name, f"ip addr add {address}/24 dev eth0")
        ns(name, "ip link set eth0 up")
        ns(name, "ip route add 224.0.0.0/4 dev eth0")


def start_relays(binary, workdir):
    make_ca(workdir)
    ns("beam-wan", "ip addr add 203.0.113.2/24 dev br0")
    for name, ip in [("x", "203.0.113.1"), ("y", "203.0.113.2")]:
        config = workdir / f"relay-{name}.toml"
        config.write_text(
            f'''http_bind_addr = "{ip}:8340"
enable_relay = true
enable_quic_addr_discovery = true
enable_metrics = false
access.shared_token = ["{RELAY_TOKEN}"]

[tls]
https_bind_addr = "{ip}:3340"
quic_bind_addr = "{ip}:7842"
hostname = "{ip}"
cert_mode = "Manual"
manual_cert_path = "{workdir}/relay.pem"
manual_key_path = "{workdir}/relay.key"
'''
        )
        sh(f"ip netns exec beam-wan {binary} --config-path {config} >{workdir}/relay-{name}.log 2>&1 &")
    time.sleep(1)


def configure(data_dir, name, relays):
    data_dir.mkdir(parents=True, exist_ok=True)
    settings = {"name": name, "relays": relays}
    if relays:
        settings["relay_ca_file"] = str(data_dir.parent / "ca.pem")
    (data_dir / "config.json").write_text(json.dumps(settings))


class Host:
    def __init__(self, namespace, data_dir, binary):
        self.namespace, self.data_dir, self.binary = namespace, data_dir, binary

    def command(self, arguments):
        quoted = " ".join(shlex.quote(str(argument)) for argument in arguments)
        return f"ip netns exec {self.namespace} sudo -n -u {USER} env BEAM_DATA_DIR={self.data_dir} {self.binary} {quoted}"

    def run(self, arguments, timeout=120):
        started = time.time()
        result = subprocess.run(["sudo", "-n", "sh", "-c", self.command(arguments)], capture_output=True, text=True, timeout=timeout)
        return result.returncode, (result.stdout + result.stderr).strip(), time.time() - started

    def spawn(self, arguments, log):
        return subprocess.Popen(["sudo", "-n", "sh", "-c", self.command(arguments)], stdout=open(log, "w"), stderr=subprocess.STDOUT)

    def stop(self):
        sh(f"ip netns pids {self.namespace} | xargs -r kill -9", check=False)


def pair(inviter, joiner, workdir):
    log = workdir / "pair.log"
    process = inviter.spawn(["pair"], log)
    for _ in range(100):
        codes = [line.strip() for line in Path(log).read_text().splitlines() if line.startswith("beam1")]
        if codes:
            break
        time.sleep(0.2)
    else:
        raise RuntimeError("no pairing code: " + Path(log).read_text()[:300])
    outcome = joiner.run(["join", codes[0]], timeout=60)
    if outcome[0] == 0:
        process.wait(timeout=20)
    return outcome
