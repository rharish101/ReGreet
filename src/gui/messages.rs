// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Message definitions for communication between the view and the model

use educe::Educe;
use greetd_ipc::Response;
use relm4::gtk::{glib::GString, prelude::*, ComboBoxText, Entry};

#[derive(Debug)]
/// Info about the current user and chosen session
pub struct UserSessInfo {
    /// The ID for the currently chosen user
    pub(super) user_id: Option<GString>,
    /// The entry text for the currently chosen user
    pub(super) user_text: GString,
    /// The ID for the currently chosen session
    pub(super) sess_id: Option<GString>,
    /// The entry text for the currently chosen session
    pub(super) sess_text: GString,
}

impl UserSessInfo {
    /// Extract session and user info from the relevant widgets.
    pub(super) fn extract(
        usernames_box: &ComboBoxText,
        username_entry: &Entry,
        sessions_box: &ComboBoxText,
        session_entry: &Entry,
    ) -> Self {
        Self {
            user_id: usernames_box.active_id(),
            user_text: username_entry.text(),
            sess_id: sessions_box.active_id(),
            sess_text: session_entry.text(),
        }
    }
}

/// The messages sent by the view to the model
#[derive(Educe)]
#[educe(Debug)]
pub enum InputMsg {
    /// Login request
    Login {
        #[educe(Debug = "ignore")]
        input: String,
        info: UserSessInfo,
    },
    /// Cancel the login request
    Cancel,
    /// The current user was changed in the GUI.
    UserChanged(UserSessInfo),
    /// Toggle manual entry of user.
    ToggleManualUser,
    /// Toggle manual entry of session.
    ToggleManualSess,
    Reboot,
    PowerOff,
}

#[derive(Debug)]
/// The messages sent to the sender to run tasks in the background
pub enum CommandMsg {
    /// Update the clock.
    UpdateTime,
    /// Clear the error message.
    ClearErr,
    /// Handle a response received from greetd
    HandleGreetdResponse(Response),
    /// Notify the greeter that a monitor was removed.
    // The Gstring is the name of the display.
    MonitorRemoved(GString),
}
