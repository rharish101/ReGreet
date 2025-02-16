// SPDX-FileCopyrightText: 2025 max-ishere <47008271+max-ishere@users.noreply.github.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

macro_rules! icons {
    ($($name:ident = $value:expr),+$(,)?) => {
        $(pub const $name: &str = $value;)+
    };
}

icons![
    POWER_MENU = self::POWEROFF,
    POWEROFF = "system-shutdown-symbolic",
    REBOOT = "system-reboot-symbolic",
    REBOOT_FIRMWARE = "application-x-firmware-symbolic",
    SUSPEND = "system-suspend-symbolic",
    HIBERNATE = "system-hibernate-symbolic",
];
