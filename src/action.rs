use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    ChangeMode(Module),
}

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize, Default, Copy)]
pub enum Module {
    #[default]
    Home,
    Cron,
}
