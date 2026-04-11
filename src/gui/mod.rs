// SPDX-FileCopyrightText: 2022 The ReGreet Authors
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The main GUI for the greeter

mod component;
mod messages;
mod model;
mod templates;
pub(crate) mod widget {
    pub mod clock;
}

pub use component::GreeterInit;
pub use model::Greeter;
