use slint::ComponentHandle;

use crate::{AppState, MainWindow, refresh};

pub(super) fn register(window: &MainWindow, state: AppState) {
    let window_weak = window.as_weak();
    let db = state.database.clone();
    let selected_program_id = state.selected_program_id.clone();
    window.on_add_exercise(move |program_id, name, sets, reps, weight, notes| {
        let db = db.borrow();
        let _ = db.add_exercise(
            program_id as i64,
            &name,
            sets as i64,
            reps as i64,
            weight as i64,
            &notes,
        );
        *selected_program_id.borrow_mut() = program_id as i64;
        if let Some(win) = window_weak.upgrade() {
            refresh::exercises(&win, &db, program_id as i64);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let selected_program_id = state.selected_program_id.clone();
    window.on_update_exercise(move |id, name, sets, reps, weight, notes| {
        let db = db.borrow();
        let _ = db.update_exercise(
            id as i64,
            &name,
            sets as i64,
            reps as i64,
            weight as i64,
            &notes,
        );
        let program_id = *selected_program_id.borrow();
        if let Some(win) = window_weak.upgrade() {
            refresh::exercises(&win, &db, program_id);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let selected_program_id = state.selected_program_id.clone();
    window.on_delete_exercise(move |id| {
        let db = db.borrow();
        let _ = db.delete_exercise(id as i64);
        let program_id = *selected_program_id.borrow();
        if let Some(win) = window_weak.upgrade() {
            refresh::exercises(&win, &db, program_id);
        }
    });
}
