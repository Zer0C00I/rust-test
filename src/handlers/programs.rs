use slint::ComponentHandle;

use crate::{refresh, AppState, MainWindow};

pub(super) fn register(window: &MainWindow, state: AppState) {
    let window_weak = window.as_weak();
    let db = state.database.clone();
    window.on_add_program(move |name, desc, weeks| {
        let db = db.borrow();
        let _ = db.add_program(&name, &desc, weeks as i64);
        if let Some(win) = window_weak.upgrade() {
            refresh::programs(&win, &db);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    window.on_update_program(move |id, name, desc, weeks| {
        let db = db.borrow();
        let _ = db.update_program(id as i64, &name, &desc, weeks as i64);
        if let Some(win) = window_weak.upgrade() {
            refresh::programs(&win, &db);
            refresh::dashboard(&win, &db);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    window.on_delete_program(move |id| {
        let db = db.borrow();
        let _ = db.delete_program(id as i64);
        if let Some(win) = window_weak.upgrade() {
            refresh::programs(&win, &db);
            refresh::dashboard(&win, &db);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let selected_program_id = state.selected_program_id.clone();
    window.on_select_program(move |program_id| {
        *selected_program_id.borrow_mut() = program_id as i64;
        let db = db.borrow();
        if let Some(win) = window_weak.upgrade() {
            refresh::exercises(&win, &db, program_id as i64);
        }
    });
}
