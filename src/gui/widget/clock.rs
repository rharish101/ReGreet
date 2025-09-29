// SPDX-FileCopyrightText: 2024 max-ishere <47008271+max-ishere@users.noreply.github.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! A [serde-configurable][`ClockConfig`] clock label widget.

use std::time::Duration;

use chrono::{Locale, Utc};
use chrono_tz::Tz;
use localzone;

use std::fmt;

use relm4::{gtk::prelude::*, prelude::*};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};
use tokio::time::sleep;

#[derive(Deserialize, Clone)]
pub struct ClockConfig {
    /// A [strftime][fmt] argument
    ///
    /// https://docs.rs/chrono/latest/chrono/format/strftime/index.html
    #[serde(alias = "fmt", default = "weekday_and_24h_time")]
    pub format: String,

    /// Amount of time between the clock's text updates
    #[serde(
        alias = "interval",
        alias = "frequency",
        with = "humantime_serde",
        default = "half_second"
    )]
    pub resolution: Duration,

    /// A timezone from the [IANA Time Zone Database](https://en.wikipedia.org/wiki/Tz_database). If the ID is invalid
    /// or [`None`], uses the system timezone.
    #[serde(alias = "tz", deserialize_with = "parse_tz", default = "system_tz")]
    pub timezone: Tz,

    /// Ask GTK to make the label this wide. This way as the text changes, the label's size can stay static.
    #[serde(default)]
    pub label_width: u32,

    /// The locale to use for time formatting (e.g. "en_US")
    #[serde(deserialize_with = "locale_deserializer", default)]
    pub locale: Locale,
}

fn weekday_and_24h_time() -> String {
    "%a %H:%M".into()
}

const fn half_second() -> Duration {
    Duration::from_millis(500)
}

fn system_tz() -> Tz {
    chrono_tz::UTC
}

const fn label_width() -> u32 {
    150
}

const fn locale() -> Locale {
    Locale::en_US
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: weekday_and_24h_time(),
            resolution: half_second(),
            timezone: system_tz(),
            label_width: label_width(),
            locale: locale(),
        }
    }
}

fn parse_tz<'de, D>(data: D) -> Result<Tz, D::Error>
where
    D: Deserializer<'de>,
{
    struct TimeZoneVisitor;

    impl Visitor<'_> for TimeZoneVisitor {
        type Value = Tz;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing an IANA Time Zone name")
        }

        fn visit_str<E>(self, time_zone_name: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // Try parsing the provided time zone string
            time_zone_name.parse::<Tz>().or_else(|_e| {
                // Fallback: try to get the system's IANA timezone via localzone
                if let Some(sys_tz_name) = localzone::get_local_zone() {
                    sys_tz_name.parse::<Tz>().map_err(|parse_err| {
                        E::custom(format!(
                            "provided TZ '{}' invalid, system TZ '{}' invalid too: {}",
                            time_zone_name, sys_tz_name, parse_err
                        ))
                    })
                } else {
                    match localzone::get_local_zone() {
                        Some(tz_name) => info!("Local system timezone: {}", tz_name),
                        None => info!("Could not determine system timezone"),
                    }
                    // Final fallback: UTC
                    Ok(chrono_tz::UTC)
                }
            })
        }
    }

    data.deserialize_any(TimeZoneVisitor)
}

fn locale_deserializer<'de, D>(deserializer: D) -> Result<Locale, D::Error>
where
    D: Deserializer<'de>,
{
    struct LocaleVisitor;

    impl Visitor<'_> for LocaleVisitor {
        type Value = Locale;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string representing a locale (e.g., 'en_US')")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // Attempt to parse the value into a Locale
            match value.parse::<Locale>() {
                Ok(locale) => Ok(locale),
                Err(_) => {
                    // If parsing fails, handle it gracefully by providing a custom error
                    Err(E::custom(format!("Invalid locale string: '{}'", value)))
                }
            }
        }
    }

    deserializer.deserialize_str(LocaleVisitor)
}

#[derive(Debug)]
pub struct Clock {
    format: String,
    timezone: Tz,
    locale: Locale,

    current_time: String,
}

/// A fixed-interval command output.
///
/// The duration between the ticks may be skewed by various factors such as the command future not being polled, so the
/// current time should be measured and formatted when the tick is recieved.
#[derive(Debug)]
pub struct Tick;

#[relm4::component(pub)]
impl Component for Clock {
    type Init = ClockConfig;
    type Input = ();
    type Output = ();
    type CommandOutput = Tick;

    view! {
        gtk::Label {
            set_width_request: label_width.min(i32::MAX as u32) as i32,

            #[watch]
            set_text: &model.current_time
        }
    }

    fn init(
        ClockConfig {
            format,
            resolution,
            timezone,
            label_width,
            locale,
        }: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        sender.command(move |sender, shutdown| {
            shutdown
                .register(async move {
                    loop {
                        if sender.send(Tick).is_err() {
                            error!("No longer updating the clock widget because `send` failed");
                            break;
                        }
                        sleep(resolution).await;
                    }
                })
                .drop_on_shutdown()
        });

        let model = Self {
            current_time: String::new(),
            format,
            timezone,
            locale,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd(&mut self, Tick: Self::CommandOutput, _: ComponentSender<Self>, _: &Self::Root) {
        let now = Utc::now().with_timezone(&self.timezone);
        let text = now.format_localized(&self.format, self.locale).to_string();

        self.current_time = text;
    }
}
