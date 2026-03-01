mod clients;
mod dashboard;
mod exercises;
mod finances;
mod print;
mod programs;
mod schedule;
mod trainers;

use crate::{AppState, MainWindow};

pub fn register_all(window: &MainWindow, state: AppState) {
    trainers::register(window, state.clone());
    programs::register(window, state.clone());
    exercises::register(window, state.clone());
    clients::register(window, state.clone());
    dashboard::register(window, state.clone());
    finances::register(window, state.clone());
    schedule::register(window, state.clone());
    print::register(window, state);
}
