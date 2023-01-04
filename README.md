[![build](https://github.com/alexVinarskis/display-switch-keycombo/workflows/build/badge.svg?branch=main)](https://github.com/alexVinarskis/display-switch-keycombo/actions)
[![GitHub license](https://img.shields.io/github/license/haimgel/display-switch)](https://github.com/alexVinarskis/display-switch-keycombo/blob/master/LICENSE)

Forked from and inspired by [`haimgel/display-switch`](https://github.com/haimgel/display-switch)

# Automate (KVM) monitor switching with a key combination
This utility allows to switch (mulitple) monitor inputs by pressing a key combination. Some monitors (eg. Dell U3419W) have built-in USB hub with multiple upstream ports, effectively making it a KVM switch without external switch button.

Altough monitor vendors provide propriatary software to switch inputs, it is often not possible to use it on Linux/MacOS, or to switch multiple monitors at once, or is generally ugly/buggy/bloatware.

This utility is efficient, cross-platform solution to switch up to 6 monitors at once, up to 4 different inputs per monitor with respective key combinations per input.

Assuming keyboard/mouse are connected via monitor's USB hub to achieve full KVM capability, this tool is supposed to be installed on all computers that could be connected to these monitors, since once the app switches inputs to other PC, pressing combo to revert will be now executed on that target PC.
 
## Platforms supported

The app should function on MacOS (Intel Macs only), Windows, and Linux.

**NOTE:** Display Switch is currently **not** working on M1 Macs: M1 series SoC support in `ddc-macos-rs` is planned but is not 
[implemented yet](https://github.com/haimgel/ddc-macos-rs/issues/2).

**NOTE:** Running on Linux requires additional packages, install via: `sudo apt install libxi-dev xorg-dev`

## Configuration

The configuration is pretty similar on all platforms:

On MacOS: the configuration file is expected in `~/Library/Preferences/display-switch-keycombo.ini` \
On Windows: the configuration file is expected in `%APPDATA%\display-switch-keycombo\display-switch-keycombo.ini` \
On Linux: the configuration file is expected in `$XDG_CONFIG_HOME/display-switch-keycombo/display-switch-keycombo.ini` or `~/.config/display-switch-keycombo/display-switch-keycombo.ini`

Configuration file settings:

```ini
combo_a = ShiftLeft ControlLeft KeyM
combo_b = ShiftLeft ControlLeft Comma
input_a = "DisplayPort1"
input_b = "Hdmi2"
```
Paramters are:
* `combo_a` ... `combo_d` are space-separated key combinations to switch to respective input. Supported keys are listed in `KEYS.md` file
* `input_a` ... `input_d` are which monitor input
to switch to, when respective key combination is detected. Supported values are `Hdmi1`, `Hdmi2`, `DisplayPort1`, `DisplayPort2`, `Dvi1`, `Dvi2`, `Vga1`.
If your monitor has an USB-C port, it's usually reported as `DisplayPort2`. Input can also be specified as a "raw" decimal or hexadecimal value: `on_usb_connect = 0x10`
* `input_a_execute` ... `input_d_execute` are optional commands to run when respective key combination is detected. See below for details.

### Different inputs on different monitors
`display-switch-keycombo` supports per-monitor configuration: add one or more monitor-specific configuration sections to set
monitor-specific inputs. For example:

```ini
combo_a = ShiftLeft ControlLeft KeyM
combo_b = ShiftLeft ControlLeft Comma

[monitor1]
monitor_id = "U3419W"
input_a = "DisplayPort1"
input_b = "Hdmi2"

[monitor2]
monitor_id = "P2418D"
input_a = "DisplayPort1"
input_b = "Hdmi1"
```

`monitor_id` specifies a case-insensitive substring to match against the monitor ID. For example, 'U3419W' would match
`DELL U3419W S/N 1144206897` monitor ID. If more than one section has a match, a first one will be used.
`input_a` ... `input_d` (if defined) take precedence over global defaults.

### Running external commands
`display-switch-keycombo` supports running external commands upon executing key combination. This configuration
can be global (runs every time specified combo is pressed) or per-monitor (runs only when
a given monitor is being switched):

```ini
input_a = "Hdmi1"
input_b = "DisplayPort2"
input_a_execute = "echo connected"
input_b_execute = "echo disconnected"

[monitor1]
monitor_id="foobar"
input_a_execute = "echo usb connected, monitor 'foobar' being switched"
input_b_execute = "'c:\\program files\\my app.exe' --parameter"
```

Notes: 
1. External applications are executed as the same user that started `display-switch-keycombo`. 
2. This program supports splitting supplied configuration into application name and parameters, but no other shell features are supported.
3. If the application path contains spaces, surround the full file path with single quotes.
4. On Windows, escape the backslashes (replace \ with \\, see the example above).

## Logging

* On MacOS: the log file is written to `/Users/USERNAME/Library/Logs/display-switch-keycombo/display-switch-keycombo.log`
* On Windows: the log file is written to `%LOCALAPPDATA%\display-switch-keycombo\display-switch-keycombo.log`
* On Linux: The log file is written to `$XDG_DATA_HOME/display-switch-keycombo/display-switch-keycombo.log`
 or `~/.local/share/display-switch-keycombo/display-switch-keycombo.log`

## Building from source

### Windows

[Install Rust](https://www.rust-lang.org/tools/install), then do `cargo build --release`

### MacOS

[Install Xcode](https://developer.apple.com/xcode/), [install Rust](https://www.rust-lang.org/tools/install), then do
`cargo build --release` 

### Linux

[Install Rust](https://www.rust-lang.org/tools/install), then do `cargo build --release`

## Running on startup

### Windows

Copy `display_switch_keycombo.exe` from `target\release` (where it was built in the previous step) to 
`%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup`.

### MacOS

```bash
  # Get your INI file in order! (see above)
  cp target/release/display_switch_keycombo /usr/local/bin/
  cp dev.haim.display-switch-keycombo.daemon.plist ~/Library/LaunchAgents/
  launchctl load ~/Library/LaunchAgents/dev.haim.display-switch-keycombo.daemon.plist
```
### Linux
Copy built executable:

```bash
  cp target/release/display_switch_keycombo /usr/local/bin/
```
Enable read/write access to i2c devices for users in `i2c` group. Run as root :

```bash
groupadd i2c
echo 'KERNEL=="i2c-[0-9]*", GROUP="i2c"' >> /etc/udev/rules.d/10-local_i2c_group.rules
udevadm control --reload-rules && udevadm trigger
```

Then add your user to the i2c group :

```
sudo usermod -aG i2c $(whoami)
```

Create a systemd unit file in your user directory (`/home/$USER/.config/systemd/user/display-switch-keycombo.service`) with contents

```
[Unit]
Description=Display switch via key combo

[Service]
ExecStart=/usr/local/bin/display_switch_keycombo
Type=simple
StandardOutput=journal
Restart=always

[Install]
WantedBy=default.target
```

Create the config file at `/home/$USER/.config/display-switch-keycombo/display-switch-keycombo.ini`.
Then enable the service with

```bash
systemctl --user daemon-reload
systemctl --user enable display-switch-keycombo.service
systemctl --user start display-switch-keycombo.service
```
