<!--
SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>

SPDX-License-Identifier: GPL-3.0-or-later
-->

<!--
SPDX-FileCopyrightText: 2021 Maximilian Moser <maximilian.moser@tuwien.ac.at>

SPDX-License-Identifier: MIT
-->

# ReGreet

A clean and customizable GTK-based [greetd](https://git.sr.ht/~kennylevinsen/greetd) greeter written in Rust using [Relm4](https://relm4.org/).
This is meant to be run under a Wayland compositor (like [Sway](https://github.com/swaywm/sway)).

It is based on [Max Moser's LightDM Elephant greeter](https://github.com/max-moser/lightdm-elephant-greeter), which is based on [Matt ~~Shultz's~~ Fischer's example LightDM greeter](https://web.archive.org/web/20210923235052/https://www.mattfischer.com/blog/archives/5).

## Screenshots

![Welcome](https://user-images.githubusercontent.com/25344287/221668247-f5193c01-2202-4739-803b-6f163da3d03b.png)
![Dropdown session menu](https://user-images.githubusercontent.com/25344287/221668339-524a731f-509c-46a7-9ceb-e6c8e92a61a0.png)
![Manual session entry](https://user-images.githubusercontent.com/25344287/221668422-ab82d10b-3167-4a31-9705-c1d066ced252.png)
![Password entry with selected user](https://user-images.githubusercontent.com/25344287/221668490-cfd231a8-bcb9-426b-ba27-783b09c29e9c.png)
![Password entry with manual user](https://user-images.githubusercontent.com/25344287/221668537-dbc8ebda-b1ec-4f71-a521-77674e2ee992.png)
![Login fail](https://user-images.githubusercontent.com/25344287/226113001-a66e8303-6d1f-4f75-b362-baccb9e07f9f.png)

These screenshots use the [Canta GTK theme](https://github.com/vinceliuice/Canta-theme) in dark mode with the Roboto font. All screenshots are provided under the [CC-BY-SA-4.0 license](https://creativecommons.org/licenses/by-sa/4.0/legalcode).

## Features
* Shows a dropdown list of existing users and X11/Wayland sessions
* Allows manual entry of username and session command
* Remembers the last authenticated user
* Automatically selects the last used session per user
* Allows setting environment variables for created sessions
* Supports customizing:
    - Background image
    - Clock
    - GTK theme
    - Dark mode
    - Icon theme
    - Cursor theme
    - Cursor blink on/off
    - Font
* Allows changing reboot & poweroff commands for different init systems
* Supports custom CSS files for further customizations
* Respects `XDG_DATA_DIRS` environment variable
* Respects fields `Hidden` and `NoDisplay` in session files
* Picks up the first found session with the same name and in the same type (X11/Wayland). This allows for overriding system-provided session files.
* Demo mode to run ReGreet without greetd for easier development.

## Requirements
* Rust 1.75.0+ (for compilation only)
* greetd
* GTK 4.0+
* A Wayland compositor (such as [Cage](https://www.hjdskes.nl/projects/cage/) or [Sway](https://swaywm.org/) or [Hyprland](https://hyprland.org/))

**Note**: Please make sure you have all requirements installed, as having a greetd greeter constantly failing isn't as much fun as it sounds.

## Installation
### Arch Linux
ReGreet is available as [greetd-regreet](https://archlinux.org/packages/extra/x86_64/greetd-regreet) in the official Arch Linux repositories, and as [greetd-regreet-git](https://aur.archlinux.org/packages/greetd-regreet-git) in the AUR.
Note that I only maintain the AUR package, and the package in the Arch repos is maintained by someone else.

Install the AUR package either by cloning the AUR repository and running `makepkg`, or by using your favourite AUR helper:
```sh
paru -S greetd-regreet-git
```

Install the package in the Arch repos as follows:
```sh
pacman -S greetd-regreet
```

### Unofficial Packages
#### NixOS
For a minimal config, add `programs.regreet.enable = true;` in your NixOS configuration file.
For users who want to configure more, they can see all the options of the module by searching for `regreet` on [NixOS Search](https://search.nixos.org/options?query=regreet).

### Manual
First, the greeter must be compiled using Cargo:
```sh
cargo build --release
```

The compilation process also configures the greeter to look for or use certain directories.
These can be changed by setting the values of certain environment variables.
These are:

Environment Variable | Default | Use
-- | -- | --
GREETD\_CONFIG\_DIR | `/etc/greetd` | The configuration directory used by greetd
STATE\_DIR | `/var/lib/regreet` | The directory used to store the ReGreet state/cache
LOG\_DIR | `/var/log/regreet` | The directory used to store logs
SESSION\_DIRS | `/usr/share/xsessions:/usr/share/wayland-sessions` | A colon (:) separated list of directories where the greeter looks for session files
X11\_CMD\_PREFIX | `startx /usr/bin/env` | The default command prefix for X11 sessions to launch the X server (see [this explanation on Reddit](https://web.archive.org/web/20240803120131/https://old.reddit.com/r/linux/comments/1c8zdcw/using_x11_window_managers_with_greetd_login/))
REBOOT\_CMD | `reboot` | The default command used to reboot the system
POWEROFF\_CMD | `poweroff` | The default command used to shut down the system
LOGIN\_DEFS\_PATHS | `/etc/login.defs:/usr/etc/login.defs` | A colon (:) separated list of `login.defs` file paths. First found is loaded.
LOGIN\_DEFS\_UID\_MIN | 1000 | Override the assumed default if `login.defs` doesnt specify `UID_MIN`.
LOGIN\_DEFS\_UID\_MAX | 60000 | Override the assumed default if `login.defs` doesnt specify `UID_MAX`.

The greeter can be installed by copying the file `target/release/regreet` to `/usr/bin` (or similar directories like `/bin`).

Optionally, to set up the log and state directories using systemd-tmpfiles, do either of the following:
* Copy the configuration given in [systemd-tmpfiles.conf](./systemd-tmpfiles.conf) to `/etc/tmpfiles.d/regreet.conf` or `/usr/lib/tmpfiles.d/regreet.conf`.
* Run the `systemd-tmpfiles` CLI:
    ```sh
    systemd-tmpfiles --create "$PWD/systemd-tmpfiles.conf"
    ```

#### GTK4 Versions
ReGreet targets GTK version 4.0 or above.
If you have higher versions of GTK, then you can enable additional features in ReGreet.
Currently, the extra features enabled are:

GTK Version | Feature Flag | Features
-- | -- | --
4.8 | `gtk4_8` | <ul><li>Changing how the background image fits the screen</li></ul>

To compile with support for a GTK version, pass the corresponding feature flag during building.
For example, to compile with GTK 4.8+ support, run:
```sh
cargo build -F gtk4_8 --release
```

To compile with full support, run:
```sh
cargo build --all-features --release
```

## Usage
### Set as Default Session
Edit the greetd config file (`/etc/greetd/config.toml`) to set ReGreet with a Wayland compositor as the default session.
For example, if using Cage:
```toml
[default_session]
command = "cage -s -mlast -- regreet"
user = "greeter"
```
The `-s` argument enables VT switching in cage (0.1.2 and newer only), which is highly recommended to prevent locking yourself out.
The `-mlast` argument tells Cage to use the last-connected monitor only, which is useful since ReGreet is a single-monitor application.

If using Sway, create a Sway config file (in a path such as `/etc/greetd/sway-config`) as follows:
```
exec "regreet; swaymsg exit"
include /etc/sway/config.d/*
```

Then, set Sway to use this config (whose path is shown here as `/path/to/custom/sway/config`) as the default greetd session:
```toml
[default_session]
command = "sway --config /path/to/custom/sway/config"
user = "greeter"
```

If using Hyprland, create a Hyprland config file (in a path such as `/etc/greetd/hyprland.conf`) as follows:
```
exec-once = regreet; hyprctl dispatch exit
misc {
    disable_hyprland_logo = true
    disable_splash_rendering = true
    disable_hyprland_guiutils_check = true
}
```

Then, set Hyprland to use this config (whose path is shown here as `/path/to/custom/hyprland/config`) as the default greetd session:
```toml
[default_session]
command = "start-hyprland -- -c /path/to/custom/hyprland/config"
user = "greeter"
```

If using Niri, create a KDL config file (in a path such as `/etc/greetd/niri.kdl`) as follows:
```kdl
spawn-sh-at-startup "regreet; niri msg action quit --skip-confirmation"
hotkey-overlay {
    skip-at-startup
}
cursor {
    // Change the theme and size of the cursor as well as set the
    // `XCURSOR_THEME` and `XCURSOR_SIZE` env variables.
    xcursor-theme "catppuccin-mocha-red-cursors"
}
```

Then, set Niri to use this config (whose path is shown here as `/path/to/custom/niri/config`) as the default greetd session:
```toml
[default_session]
command = "niri --config /path/to/custom/niri/config"
user = "greeter"
```

Restart greetd to use the new config.

#### Startup delays
If you find that ReGreet takes too much time to start up, you may be affected by this: [swaywm/sway/wiki#gtk-applications-take-20-seconds-to-start](https://github.com/swaywm/sway/wiki#gtk-applications-take-20-seconds-to-start).
See this link for the fix.
Alternatively, the solution proposed in [issue #34](https://github.com/rharish101/ReGreet/issues/34) may resolve it.

As another option, you can disable portals by exporting environment variables for the Wayland compositor launched for ReGreet.
Simply prepend `env GTK_USE_PORTAL=0 GDK_DEBUG=no-portals` to the start of the default session command in `greetd.toml`.
For example, with Cage, the session command would be:
```toml
[default_session]
command = "env GTK_USE_PORTAL=0 GDK_DEBUG=no-portals cage -s -mlast -- regreet"
```

If using Hyprland, you can instead append the following lines to the Hyprland config for ReGreet:
```
env = GTK_USE_PORTAL,0
env = GDK_DEBUG,no-portals
```

### Configuration
The configuration file must be in the [TOML](https://toml.io/) format.
By default, it is named `regreet.toml`, and located in the greetd configuration directory specified during compilation (`/etc/greetd/` by default).
You can use a config file in a different location with the `--config` argument as follows:
```sh
regreet --config /path/to/custom/regreet/config.toml
```

A sample configuration is provided along with sample values for all available options in [`regreet.sample.toml`](regreet.sample.toml).
Currently, the following can be configured:
* Background image
* How the background image fits the screen (needs GTK 4.8+ support compiled)
* Environment variables for created sessions
* Greeting message
* Clock
* GTK theme
* Dark mode
* Icon theme
* Cursor theme
* Cursor blink on/off
* Font
* Reboot command
* Shut down command
* X11 command prefix (see [this explanation on Reddit](https://web.archive.org/web/20240803120131/https://old.reddit.com/r/linux/comments/1c8zdcw/using_x11_window_managers_with_greetd_login/))

**NOTE:** For configuring other essential features, such as the keyboard layout/mapping, the choice of monitor to use, etc., please check out the configuration options for the wayland compositor that you are using to run ReGreet.
For example, if you use Cage, check out the [Cage wiki](https://github.com/cage-kiosk/cage/wiki/Configuration).
If you use Sway, check out the [Sway wiki](https://github.com/swaywm/sway/wiki#configuration).
If you use Hyprland, check out the [Hyprland wiki](https://wiki.hyprland.org/).

### Custom CSS
ReGreet supports loading CSS files to act as a custom global stylesheet.
This enables one to do further customizations above what ReGreet supports through the config file.

By default, the custom CSS file is named `regreet.css`, and located in the greetd configuration directory specified during compilation (`/etc/greetd/` by default).
To load a custom CSS stylesheet from a different location, pass the `-s` or `--style` CLI argument as follows:
```sh
regreet --style /path/to/custom.css
```

Please refer to the GTK4 docs on [CSS in GTK](https://docs.gtk.org/gtk4/css-overview.html) and [GTK CSS Properties](https://docs.gtk.org/gtk4/css-properties.html) to learn how to style a GTK4 app using CSS.
For a general reference on CSS, please refer to the [MDN web docs](https://developer.mozilla.org/en-US/docs/Web/CSS/Syntax).

**Tip:** You might want to use [demo mode](#demo-mode) to test out your CSS before making it permanent.

### Changing Reboot/Shut Down Commands
The default reboot and shut down commands use the `reboot` and `poweroff` binaries, which are present on most Linux systems.
However, since the recommended way of using ReGreet is to avoid running it as root, the `reboot`/`poweroff` commands might not work on systems where superuser access is needed to run these commands.
In this case, if there is another command to reboot or shut down the system without superuser access, these commands can be set in the config file under the `[commands]` section.

For example, to use `loginctl reboot` as the reboot command, use the following config:
```toml
[commands]
reboot = [ "loginctl", "reboot" ]
```
Here, each command needs to be separated into a list containing the main command, followed by individual arguments.

These commands can also be specified during compilation using the `REBOOT_CMD` and `POWEROFF_CMD` environment variables.

### Logging and Caching
The state is are stored in `/var/lib/regreet/state.toml` (configurable during installation).
It contains the last authenticated user and the last used session per user, which are automatically selected on next login.
If the greeter is unable to write to this file, then it reverts to the default behaviour.

By default, the logs are stored in `/var/log/regreet/log` (configurable during installation).
You can use a log file in a different location with the `--logs` argument as follows:
```sh
regreet --logs /path/to/custom/regreet/logs
```

Once the log file reaches a limit, it is compressed and rotated to `log.X.gz` in the same directory, where `X` is the index of the log file.
The higher the index, the older the log file.
After reaching a limit, the oldest log file is removed.

If the greeter is unable to write to this file or create files in the log directory, then it logs to stdout.
You can also print the logs to stdout in addition to the log file, with the `--verbose` argument as follows:
```sh
regreet --verbose
```

The recommended configuration is to run greetd greeters as a separate user (`greeter` in the above examples).
This can lead to insufficient permissions for either creating the state/log directories, or writing to them.
To make use of the caching and logging features, please create the directories manually with the correct permissions, if not done during installation with systemd-tmpfiles.

## Contributing
[pre-commit](https://pre-commit.com/) is used for managing hooks that run before each commit (such as clippy), to ensure code quality.
Thus, this needs to be set up only when one intends to commit changes to git.

Firstly, [install pre-commit](https://pre-commit.com/#installation) itself.
Next, install pre-commit hooks:
```sh
pre-commit install
```

Now, pre-commit should ensure that the code passes all linters locally before committing.
This will save time when creating PRs, since these linters also run in CI, and thus fail code that hasn't been linted well.

### Demo mode
To aid development, a "demo" mode is included within ReGreet that runs ReGreet independent of greetd.
Simply run ReGreet as follows:
```sh
regreet --demo
```

Since the demo mode doesn't use greetd, authentication is done using hardcoded credentials within the codebase.
These credentials are logged with the warning log-level, so that you don't have to read the source code.

## Licenses
This repository uses [REUSE](https://reuse.software/) to document licenses.
Each file either has a header containing copyright and license information, or has an entry in the [TOML file](https://reuse.software/spec-3.3/#reusetoml) at [REUSE.toml](./REUSE.toml).
The license files that are used in this project can be found in the [LICENSES](./LICENSES) directory.

A copy of the GPL-3.0-or-later license is placed in [LICENSE](./LICENSE), to signify that it constitutes the majority of the codebase, and for compatibility with GitHub.
