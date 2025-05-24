// SPDX-FileCopyrightText: 2022 Harish Rajagopal <harish.rajagopals@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Templates for various GUI components
#![allow(dead_code)] // Silence dead code warnings for UI code that isn't dead

use gtk::prelude::*;
use relm4::{gtk, RelmWidgetExt, WidgetTemplate};

use crate::gui::GAP;

/// Label for an entry/combo box
#[relm4::widget_template(pub)]
impl WidgetTemplate for EntryLabel {
    view! {
        gtk::Label {
            set_width_request: 100,
            set_xalign: 1.0,
        }
    }
}

/// Main UI of the greeter
#[relm4::widget_template(pub)]
impl WidgetTemplate for Ui {
    view! {
        gtk::Overlay {
            /// Background image
            #[name = "background"]
            gtk::Picture,

            /// Main login box
            add_overlay = &gtk::Frame {
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                add_css_class: "background",

                gtk::Grid {
                    set_column_spacing: GAP as u32,
                    set_margin_bottom: GAP,
                    set_margin_end: GAP,
                    set_margin_start: GAP,
                    set_margin_top: GAP,
                    set_row_spacing: GAP as u32,
                    set_width_request: 500,

                    /// Widget to display messages to the user
                    #[name = "message_label"]
                    attach[0, 0, 3, 1] = &gtk::Label {
                        set_margin_bottom: GAP,

                        // Format all messages in boldface.
                        #[wrap(Some)]
                        set_attributes = &gtk::pango::AttrList {
                            insert: {
                                let mut font_desc = gtk::pango::FontDescription::new();
                                font_desc.set_weight(gtk::pango::Weight::Bold);
                                gtk::pango::AttrFontDesc::new(&font_desc)
                            },
                        },
                    },

                    #[template]
                    attach[0, 1, 1, 1] = &EntryLabel {
                        set_label: "User:",
                        set_height_request: 45,
                    },

                    /// Label for the sessions widget
                    #[name = "session_label"]
                    #[template]
                    attach[0, 2, 1, 1] = &EntryLabel {
                        set_label: "Session:",
                        set_height_request: 45,
                    },

                    /// Widget containing the usernames
                    #[name = "usernames_box"]
                    attach[1, 1, 1, 1] = &gtk::ComboBoxText { set_hexpand: true },

                    /// Widget where the user enters the username
                    #[name = "username_entry"]
                    attach[1, 1, 1, 1] = &gtk::Entry { set_hexpand: true },

                    /// Widget containing the sessions
                    #[name = "sessions_box"]
                    attach[1, 2, 1, 1] = &gtk::ComboBoxText,

                    /// Widget where the user enters the session
                    #[name = "session_entry"]
                    attach[1, 2, 1, 1] = &gtk::Entry,

                    /// Label for the password widget
                    #[name = "input_label"]
                    #[template]
                    attach[0, 2, 1, 1] = &EntryLabel {
                        set_height_request: 45,
                    },

                    /// Widget where the user enters a secret
                    #[name = "secret_entry"]
                    attach[1, 2, 1, 1] = &gtk::PasswordEntry { set_show_peek_icon: true },

                    /// Widget where the user enters something visible
                    #[name = "visible_entry"]
                    attach[1, 2, 1, 1] = &gtk::Entry,

                    /// Button to toggle manual user entry
                    #[name = "user_toggle"]
                    attach[2, 1, 1, 1] = &gtk::ToggleButton {
                        set_icon_name: "document-edit-symbolic",
                        set_tooltip_text: Some("Manually enter username"),
                    },

                    /// Button to toggle manual session entry
                    #[name = "sess_toggle"]
                    attach[2, 2, 1, 1] = &gtk::ToggleButton {
                        set_icon_name: "document-edit-symbolic",
                        set_tooltip_text: Some("Manually enter session command"),
                    },

                    /// Collection of action buttons (eg. Login)
                    attach[1, 3, 2, 1] = &gtk::Box {
                        set_halign: gtk::Align::End,
                        set_spacing: GAP,

                        /// Button to cancel password entry
                        #[name = "cancel_button"]
                        gtk::Button {
                            set_focusable: true,
                            set_label: "Cancel",
                        },

                        /// Button to enter the password and login
                        #[name = "login_button"]
                        gtk::Button {
                            set_focusable: true,
                            set_label: "Login",
                            set_receives_default: true,
                            add_css_class: "suggested-action",
                        },
                    },
                },
            },

            add_overlay = &gtk::CenterBox {
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Start,
                set_hexpand:true,

                /// Clock widget
                #[name = "clock_frame"]
                #[wrap(Some)]
                set_center_widget = &gtk::Frame {
                    add_css_class: "background",

                    // Make it fit cleanly onto the top edge of the screen.
                    inline_css: "
                        border-top-right-radius: 0px;
                        border-top-left-radius: 0px;
                        border-top-width: 0px;
                    ",
                },

                #[name = "panel_end"]
                #[wrap(Some)]
                set_end_widget = &gtk::Box {
                    set_hexpand: true,
                    set_halign: gtk::Align::End,
                },
            },


            /// Collection of widgets appearing at the bottom
            add_overlay = &gtk::Box::new(gtk::Orientation::Vertical, GAP) {
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::End,
                set_margin_bottom: GAP,

                gtk::Frame {
                    /// Notification bar for error messages
                    #[name = "error_info"]
                    gtk::InfoBar {
                        // During init, the info bar closing animation is shown. To hide that, make
                        // it invisible. Later, the code will permanently make it visible, so that
                        // `InfoBar::set_revealed` will work properly with animations.
                        set_visible: false,
                        set_message_type: gtk::MessageType::Error,

                        /// The actual error message
                        #[name = "error_label"]
                        gtk::Label {
                            set_halign: gtk::Align::Center,
                            set_margin_top: 10,
                            set_margin_bottom: 10,
                            set_margin_start: 10,
                            set_margin_end: 10,
                        },
                    }
                },
            },
        }
    }
}
