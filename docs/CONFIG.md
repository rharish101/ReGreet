<!--
SPDX-FileCopyrightText: 2024 max-ishere

SPDX-License-Identifier: GPL-3.0-or-later
-->
# ReGreet configuration docs

## Clock

The clock widget can be customized to specify the 
[time display format](https://docs.rs/jiff/0.1.14/jiff/fmt/strtime/index.html#conversion-specifications),
how often to update the clock, overrides to the system timezone and a custom fixed label width so that the non-monospace
font does not cause annoying width changes in the UI.

```toml
[widget.clock]
# strftime format argument
format = "%a %H:%M"

# How often to update the text
resolution = "500ms"

# Override system timezone (IANA Time Zone Database name, aka /etc/zoneinfo path)
# Remove to use the system time zone.
timezone = "America/Chicago"

# Ask GTK to make the label at least this wide. This helps keeps the parent element layout and width consistent.
# Experiment with different widths, the interpretation of this value is entirely up to GTK.
label_width = 150
```

## Power menu

The first step in configuring the power menu is to select the backend to use. The backend is specified as part of the
path under `widget.power-menu.<backend>`. Please note that only 1 backend can be active at a time.

For your convenience, most backends allow you to specify an action that is automatically translated into the supported
languages. The correct icon is also selected. Actions that will poweroff your system will ask for confirmation before
they are executed. The actions are listed in the table below.

| Config value    | Confirmation required |
|-----------------|-----------------------|
| poweroff        | Yes                   |
| halt            | Yes                   |
| reboot          | Yes                   |
| reboot-firmware | Yes                   |
| suspend         | No                    |
| hibernate       | No                    |
| hybrid-sleep    | No                    |

The convention most backends' configs follow is having a single `actions` key, which is an array of actions. This simple
structure allows you to easily add, remove, and reorder the buttons in the menu without having to verbosely specify all
the attributes like you can do with the [custom](#custom) backend.

### Systemd

Machines using Systemd as the init system can specify the following line to use the default configuration.

```toml
[widget.power-menu.systemd]
```

The displayed actions can be changed using the `actions` key. [See above](#power-menu) for more details

```toml
[widget.power-menu.systemd]
actions = ["suspend", "poweroff", "reboot-firmware"]
```

### Unix

This backend should work on most systems as it does not use commands shipped with any specific init system. Instead, the
Linux `shutdown` command is used.

```toml
[widget.power-menu.unix]
```

> [!NOTE]
> The only actions supported by this backend are poweroff, reboot, and halt.
>
> When other values are specified they are logged as warnings and silently skipped.

```toml
[widget.power-menu.systemd]
actions = ["poweroff", "reboot", "halt"]
```

### Custom

The custom backend allows you to fully customize the power menu. If none of the other backends suit your needs or
require additional customizations, consider using this backend.

It is necesary to set the `backend` key to the backend that you are implementing. The backend string can have any value
and is only used as part of the label in the power menu. The actions can be specified in the `commands` key.

The power menu button (as well as `commands`) has the following fields:

- `String` Label
- `String` Icon (optional)
- `bool` Should this entry be confirmed?
- `Vec<String>` Command to execute

The label, icon and confirmation can be inferred by ReGreet automatically if you set the `action` field.

```toml
[widget.power-menu.custom]
backend = "Custom" # Required value

[[widget.power-menu.custom.commands]]
action = "poweroff"
command = ["shutdown", "--now"]
# `action` Infers:
# label = "..." ## NOTE: label and action are mutually exclusive
# icon = "..."
# confirm = ...
```

Overrides (that are different from the action defaults) can be specified for the icon and the confirm, but not the
label. The logic is that if you are specifying a custom label, you are likely also specifying a custom command that
ReGreet does not know how to interpret into the inferable fields.

The icon can be hidden altogether by setting it to an empty string.

```toml
[[widget.power-menu.custom.commands]]
action = "poweroff"
command = ["shutdown", "--now"]
icon = "" # no icon
confirm = false
# Infers:
# label = Poweroff" ## NOTE: label and action are mutually exclusive
```
