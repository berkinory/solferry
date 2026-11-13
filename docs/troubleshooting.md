# Troubleshooting

## A device doesn't show up under Nearby

1. **Same network?** Both devices need to be on the same Wi-Fi or wired network, or the same Tailscale network. Guest networks and some office or school networks keep devices apart from each other.
2. **Let others find me** has to be on for the device you're looking for. Paired devices find each other either way.
3. **Local network permission.** On macOS and iOS, Solferry needs it to find anything nearby. Check System Settings or Settings, Privacy & Security, Local Network.
4. **Firewall.** Allow incoming UDP on port 53318. On Windows, allow Solferry on private networks when Windows Firewall asks. On Linux with `ufw`:

   ```sh
   sudo ufw allow 53318/udp
   ```

5. **Try the address.** Under **Use a code**, enter the other device's IP address, which it shows on that same screen. From a terminal, `solferry nearby --address 192.168.1.20` asks that one address.

If the two devices are on different networks, see the next section.

## Reaching a device on another network

Without help, devices on different networks usually can't reach each other. Two ways around it:

- Put both on the same [Tailscale](https://tailscale.com) network. Solferry finds Tailscale devices by itself, with no setup.
- [Run your own relay](relay.md). Pair once, and both devices use it from then on.

`solferry relay check` tests a relay and says what to fix: the DNS name, the token or the UDP port.

## Transfers are slow or keep reconnecting

- **The route.** The transfer toast shows whether a transfer is direct or going through a relay. Through a relay it can only be as fast as the relay's connection, and your **Relay speed limit** in Settings, if you set one.
- **Wi-Fi.** A weak signal on either end limits everything. Try both devices on the same access point, or one of them wired.
- **Reconnecting** means the connection dropped and Solferry is getting it back. The transfer continues from where it stopped, and nothing received is lost.

## I don't get notifications

Solferry needs notification permission to tell you about incoming files, texts and pairing requests. The Permissions card in Settings shows whether it's allowed, with a button to the right system setting.

On macOS, notifications are off for apps you haven't allowed: check System Settings, Notifications, Solferry. Accept and Decline appear when you hover over the notification or open it.

On Android, turning off **Receive in the background** means nothing arrives while the app is closed.

## The other device asks every time

That's on purpose. A paired device still asks until you choose **Always accept** on one of its offers, on the receiving device. See [Trust and privacy](trust.md).

## A send says it's waiting

The device is offline, so the send is waiting in **Waiting to send** on the home screen. It goes out when that device comes back while Solferry runs on yours, or gives up after 7 days. Cancel it there if you don't want it anymore.

## Windows warns about an unknown publisher

The installer isn't code signed yet. Choose **More info**, then **Run anyway**. The download and its hash are listed on the [releases page](https://github.com/berkinory/solferry/releases).

## Still stuck

Run the command line tool with logs and open an issue with what it prints, leaving out anything private:

```sh
SOLFERRY_LOG=info solferry receive
```

Issues: [github.com/berkinory/solferry/issues](https://github.com/berkinory/solferry/issues)
