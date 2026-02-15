use std::{env, fs, process::Command};

use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::Matcher;
use serde::Deserialize;

use self::history::History;

mod history;

#[derive(Deserialize)]
struct HistoryConfig {
    capacity: usize,
}

#[derive(Deserialize)]
struct Config {
    prefix: String,
    shell: Option<String>,
    history: Option<HistoryConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            prefix: ":sh".to_string(),
            shell: None,
            history: None,
        }
    }
}

#[derive(Default)]
struct State {
    config: Config,
    history: Option<History>,
}

#[init]
fn init(config_dir: RString) -> State {
    match fs::read_to_string(format!("{}/shell.ron", config_dir)) {
        Ok(content) => {
            let config: Config = ron::from_str(&content).unwrap_or_default();

            let history = config.history.as_ref().map(History::new);

            State { config, history }
        }
        Err(_) => State::default(),
    }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Shell".into(),
        icon: "utilities-terminal".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, state: &State) -> RVec<Match> {
    let config = &state.config;
    if input.starts_with(&config.prefix) {
        let (_, command) = input.split_once(&config.prefix).unwrap();
        let command = command.trim();
        if !command.is_empty() {
            let matches = if let Some(history) = &state.history {
                let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
                let matches = Atom::new(
                    command,
                    CaseMatching::Ignore,
                    Normalization::Smart,
                    AtomKind::Fuzzy,
                    false,
                )
                .match_list(&history.elements, &mut matcher)
                .into_iter()
                .map(|(s, _)| s.as_str())
                .collect();
                matches
            } else {
                vec![command]
            };

            std::iter::once(command)
                .chain(matches.into_iter())
                .map(|cmd| Match {
                    title: cmd.into(),
                    description: ROption::RSome(
                        config
                            .shell
                            .clone()
                            .unwrap_or_else(|| {
                                env::var("SHELL").unwrap_or_else(|_| {
                                    "The shell could not be determined!".to_string()
                                })
                            })
                            .into(),
                    ),
                    use_pango: false,
                    icon: ROption::RNone,
                    id: ROption::RNone,
                })
                .collect::<Vec<Match>>()
                .into()
        } else {
            RVec::new()
        }
    } else {
        RVec::new()
    }
}

#[handler]
fn handler(selection: Match, state: &mut State) -> HandleResult {
    if let Some(history) = state.history.as_mut() {
        if let Err(err) = history.push(selection.title.clone().into_string()) {
            eprintln!("[shell] Failed to push command to history: {:?}", err);
        }
    }

    if let Err(why) = Command::new(selection.description.unwrap().as_str())
        .arg("-c")
        .arg(selection.title.as_str())
        .spawn()
    {
        eprintln!("[shell] Failed to run command: {}", why);
    }

    HandleResult::Close
}
