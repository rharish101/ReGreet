// SPDX-FileCopyrightText: 2022 The ReGreet Authors
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use gettextrs::gettext;

use crate::constants::{CONFIG_PATH, CSS_PATH, LOG_PATH};

// The gettext calls are in dedicated functions rather than in the attributes
// below: xgettext skips lines starting with '#', so a call written directly
// in a derive attribute is never extracted.

fn about_text() -> String {
    gettext("Clean and customizable greeter for greetd")
}

fn logs_help() -> String {
    gettext("The path to the log file")
}

fn log_level_help() -> String {
    gettext("The verbosity level of the logs")
}

fn verbose_help() -> String {
    gettext("Output all logs to stdout")
}

fn config_help() -> String {
    gettext("The path to the config file")
}

fn style_help() -> String {
    gettext("The path to the custom CSS stylesheet")
}

fn demo_help() -> String {
    gettext("Run in demo mode")
}

#[derive(Clone, Debug, ValueEnum)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Parser, Debug)]
#[command(author, version, about = about_text())]
pub struct Args {
    #[arg(short = 'l', long, value_name = "PATH", default_value = LOG_PATH, help = logs_help())]
    pub logs: PathBuf,

    #[arg(short = 'L', long, value_name = "LEVEL", default_value = "info", help = log_level_help())]
    pub log_level: LogLevel,

    #[arg(short, long, help = verbose_help())]
    pub verbose: bool,

    #[arg(short, long, value_name = "PATH", default_value = CONFIG_PATH, help = config_help())]
    pub config: PathBuf,

    #[arg(short, long, value_name = "PATH", default_value = CSS_PATH, help = style_help())]
    pub style: PathBuf,

    #[arg(long, help = demo_help())]
    pub demo: bool,
}
