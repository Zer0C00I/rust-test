use slint::ComponentHandle;

use crate::{refresh, AppState, MainWindow};

pub(super) fn register(window: &MainWindow, state: AppState) {
    let window_weak = window.as_weak();
    let db = state.database.clone();
    let expanded_program_id = state.expanded_program_id.clone();
    window.on_expand_card(move |cp_id| {
        *expanded_program_id.borrow_mut() = cp_id as i64;
        let db = db.borrow();
        if let Some(win) = window_weak.upgrade() {
            let checkin_rows = db
                .get_checkins_for_client_program(cp_id as i64)
                .unwrap_or_default();
            refresh::checkins(&win, &checkin_rows);
            refresh::exercise_completions(&win, &db, cp_id as i64);
            refresh::exercise_week_completions(&win, &db, cp_id as i64);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    window.on_toggle_exercise_week(move |cp_id, exercise_id, checkin_id| {
        let db = db.borrow();
        let _ = db.toggle_exercise_week(cp_id as i64, exercise_id as i64, checkin_id as i64);
        if let Some(win) = window_weak.upgrade() {
            refresh::exercise_week_completions(&win, &db, cp_id as i64);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    window.on_toggle_checkin(move |checkin_id| {
        let db = db.borrow();
        let _ = db.toggle_checkin(checkin_id as i64);
        if let Some(win) = window_weak.upgrade() {
            let cp_id = db.get_client_program_for_checkin(checkin_id as i64);
            let checkin_rows = db
                .get_checkins_for_client_program(cp_id)
                .unwrap_or_default();
            refresh::checkins(&win, &checkin_rows);
        }
    });
}
