//! Message definitions for communication between the view and the model
use std::fmt::{Debug, Error as FmtError, Formatter};

use relm4::gtk::{glib, prelude::*, ComboBoxText};

#[derive(Debug)]
pub struct UserSessInfo {
    pub(super) user_id: Option<glib::GString>,
    pub(super) user_text: Option<glib::GString>,
    pub(super) sess_id: Option<glib::GString>,
    pub(super) sess_text: Option<glib::GString>,
}

impl UserSessInfo {
    pub(super) fn extract(usernames_box: &ComboBoxText, sessions_box: &ComboBoxText) -> Self {
        Self {
            user_id: usernames_box.active_id(),
            user_text: usernames_box.active_text(),
            sess_id: sessions_box.active_id(),
            sess_text: sessions_box.active_text(),
        }
    }
}

pub enum InputMsg {
    Login {
        password: String,
        info: UserSessInfo,
    },
    Cancel,
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
pub enum CommandMsg {
    UpdateTime,
    ClearErr,
    Noop,
}
