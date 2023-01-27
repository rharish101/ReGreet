//! Message definitions for communication between the view and the model
use std::fmt::{Debug, Error as FmtError, Formatter};

use relm4::gtk::{glib, prelude::*, ComboBoxText};

#[derive(Debug)]
/// Info about the current user and chosen session
pub struct UserSessInfo {
    /// The ID for the currently chosen user
    pub(super) user_id: Option<glib::GString>,
    /// The entry text for the currently chosen user
    pub(super) user_text: Option<glib::GString>,
    /// The ID for the currently chosen session
    pub(super) sess_id: Option<glib::GString>,
    /// The entry text for the currently chosen session
    pub(super) sess_text: Option<glib::GString>,
}

impl UserSessInfo {
    /// Extract session and user info from the relevant widgets.
    pub(super) fn extract(usernames_box: &ComboBoxText, sessions_box: &ComboBoxText) -> Self {
        Self {
            user_id: usernames_box.active_id(),
            user_text: usernames_box.active_text(),
            sess_id: sessions_box.active_id(),
            sess_text: sessions_box.active_text(),
        }
    }
}

/// The messages sent by the view to the model
pub enum InputMsg {
    /// Login request
    Login {
        password: String,
        info: UserSessInfo,
    },
    /// Cancel the login request
    Cancel,
    /// The current user was changed in the GUI.
    UserChanged(UserSessInfo),
    Reboot,
    PowerOff,
}

// Manually implement Debug so that the password isn't accidentally logged.
impl Debug for InputMsg {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Self::Login { password: _, info } => {
                // Skip the password field.
                f.debug_struct("Login").field("info", info).finish()
            }
            Self::Cancel => f.write_str("Cancel"),
            Self::UserChanged(info) => f.debug_tuple("UserChanged").field(info).finish(),
            Self::Reboot => f.write_str("Reboot"),
            Self::PowerOff => f.write_str("PowerOff"),
        }
    }
}

#[derive(Debug)]
/// The messages sent to the sender to run tasks in the background
pub enum CommandMsg {
    /// Update the clock.
    UpdateTime,
    /// Clear the error message.
    ClearErr,
    /// Do nothing.
    Noop,
}
