// SPDX-FileCopyrightText: 2024 max-ishere <47008271+max-ishere@users.noreply.github.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! A [serde-configurable][`ClockConfig`] clock label widget.

use std::time::Duration;

use jiff::{fmt::strtime::format, tz::TimeZone, Timestamp, Zoned};
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
    /// [fmt]: jiff::fmt::strtime
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
    pub timezone: TimeZone,

    /// Ask GTK to make the label this wide. This way as the text changes, the label's size can stay static.
    #[serde(default)]
    pub label_width: u32,
}

fn weekday_and_24h_time() -> String {
    "%a %H:%M".into()
}

const fn half_second() -> Duration {
    Duration::from_millis(500)
}

fn system_tz() -> TimeZone {
    TimeZone::system()
}

const fn label_width() -> u32 {
    150
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: weekday_and_24h_time(),
            resolution: half_second(),
            timezone: system_tz(),
            label_width: label_width(),
        }
    }
}

fn parse_tz<'de, D>(data: D) -> Result<TimeZone, D::Error>
where
    D: Deserializer<'de>,
{
    struct TimeZoneVisitor;
    impl Visitor<'_> for TimeZoneVisitor {
        type Value = TimeZone;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string containing an IANA Time Zone name")
        }

        fn visit_str<E>(self, time_zone_name: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(TimeZone::get(time_zone_name).unwrap_or_else(|e| {
                error!("Invalid timezone '{time_zone_name}' in the config: {e}");
                TimeZone::system()
            }))
        }
    }

    data.deserialize_any(TimeZoneVisitor)
}

#[derive(Debug)]
pub struct Clock {
    format: String,
    timezone: TimeZone,

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
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd(&mut self, Tick: Self::CommandOutput, _: ComponentSender<Self>, _: &Self::Root) {
        let now = Zoned::new(Timestamp::now(), self.timezone.clone());

        let text = match jiff::fmt::strtime::format(&self.format, &now) {
            Ok(str) => str,
            Err(_) => format(weekday_and_24h_time(), &now)
                .unwrap_or_else(|_| "Time formatting error.".into()),
        };

        self.current_time = text;
    }
}
