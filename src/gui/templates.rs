//! Templates for various GUI components
use gtk::prelude::*;
use relm4::{gtk, RelmWidgetExt, WidgetTemplate};

/// Button that ends the greeter
#[relm4::widget_template(pub)]
impl WidgetTemplate for EndButton {
    view! {
        gtk::Button {
            set_focusable: true,
            add_css_class: "destructive-action",
        }
    }
}

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

/// The main UI of the greeter
#[relm4::widget_template(pub)]
impl WidgetTemplate for Ui {
    view! {
        gtk::Overlay {
            /// The background image
            #[name = "background"]
            gtk::Picture,

            add_overlay = &gtk::Frame {
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                inline_css: "background-color: @theme_bg_color",

                gtk::Grid {
                    set_column_spacing: 15,
                    set_margin_bottom: 15,
                    set_margin_end: 15,
                    set_margin_start: 15,
                    set_margin_top: 15,
                    set_row_spacing: 15,
                    set_width_request: 500,

                    /// The widget to display messages to the user
                    #[name = "message_label"]
                    attach[1, 0, 1, 1] = &gtk::Label {
                        set_hexpand: true,
                        set_margin_bottom: 15,
                        set_xalign: 0.0,

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
                    attach[0, 1, 1, 1] = &EntryLabel { set_label: "User:" },

                    /// The label for the sessions widget
                    #[name = "session_label"]
                    #[template]
                    attach[0, 2, 1, 1] = &EntryLabel { set_label: "Session:" },

                    /// The widget containing the usernames
                    #[name = "usernames_box"]
                    attach[1, 1, 1, 1] = &gtk::ComboBoxText::with_entry(),

                    /// The widget containing the sessions
                    #[name = "sessions_box"]
                    attach[1, 2, 1, 1] = &gtk::ComboBoxText::with_entry(),

                    /// The label for the password widget
                    #[name = "password_label"]
                    #[template]
                    attach[0, 2, 1, 1] = &EntryLabel { set_label: "Password:" },

                    /// The widget where the user enters the password
                    #[name = "password_entry"]
                    attach[1, 2, 1, 1] = &gtk::PasswordEntry { set_show_peek_icon: true },

                    attach[1, 3, 1, 1] = &gtk::Box {
                        set_halign: gtk::Align::End,
                        set_spacing: 15,

                        /// The button to cancel password entry
                        #[name = "cancel_button"]
                        gtk::Button {
                            set_focusable: true,
                            set_label: "Cancel",
                        },

                        /// The button to enter the password and login
                        #[name = "login_button"]
                        gtk::Button {
                            set_focusable: true,
                            set_label: "Login",
                            set_receives_default: true,
                            add_css_class: "suggested-action",
                        },
                    },
                }
            },

            add_overlay = &gtk::Frame {
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Start,
                inline_css: "
                    border-top-right-radius: 0px;
                    border-top-left-radius: 0px;
                    border-top-width: 0px;
                    background-color: @theme_bg_color;
                ",

                /// The datetime label
                #[name = "datetime_label"]
                gtk::Label { set_width_request: 150 },
            },

            add_overlay = &gtk::Box {
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::End,
                set_homogeneous: true,
                set_margin_bottom: 15,
                set_spacing: 15,

                /// The button to reboot
                #[name = "reboot_button"]
                #[template]
                EndButton { set_label: "Reboot" },

                /// The button to power-off
                #[name = "poweroff_button"]
                #[template]
                EndButton { set_label: "Power Off" },
            },
        }
    }
}
