// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The main GUI for the greeter

mod component;
mod messages;
mod model;
mod templates;
pub(crate) mod widget {
    pub mod clock;
    pub mod power_menu;
}
pub mod icons;

pub use component::GreeterInit;
pub use model::Greeter;
