use slint::ComponentHandle;

use crate::{AppState, MainWindow, db, refresh};

pub(super) fn register(window: &MainWindow, state: AppState) {
    let window_weak = window.as_weak();
    let db = state.database.clone();
    window.on_add_client(move |name, phone, email, notes, trainer_idx| {
        let db = db.borrow();
        let trainer_id = trainer_idx_to_id(&db, trainer_idx);
        let _ = db.add_client(&name, &phone, &email, &notes, trainer_id);
        if let Some(win) = window_weak.upgrade() {
            refresh::clients(&win, &db);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    window.on_update_client(move |id, name, phone, email, notes, trainer_idx| {
        let db = db.borrow();
        let trainer_id = trainer_idx_to_id(&db, trainer_idx);
        let _ = db.update_client(id as i64, &name, &phone, &email, &notes, trainer_id);
        if let Some(win) = window_weak.upgrade() {
            refresh::clients(&win, &db);
            refresh::dashboard(&win, &db);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let current_year = state.current_year.clone();
    let current_month = state.current_month.clone();
    window.on_delete_client(move |id| {
        let db = db.borrow();
        let _ = db.delete_client(id as i64);
        if let Some(win) = window_weak.upgrade() {
            let year = *current_year.borrow();
            let month = *current_month.borrow();
            refresh::clients(&win, &db);
            refresh::dashboard(&win, &db);
            refresh::visits(&win, &db, year, month);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let selected_client_id = state.selected_client_id.clone();
    window.on_select_client(move |client_id| {
        *selected_client_id.borrow_mut() = client_id as i64;
        let db = db.borrow();
        if let Some(win) = window_weak.upgrade() {
            refresh::client_programs(&win, &db, client_id as i64);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    window.on_assign_program(move |client_id, program_idx| {
        let db = db.borrow();
        let programs = db.get_programs().unwrap_or_default();
        if let Some(program) = programs.get(program_idx as usize) {
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            let _ = db.assign_program(client_id as i64, program.id, &today);
            if let Some(win) = window_weak.upgrade() {
                refresh::client_programs(&win, &db, client_id as i64);
                refresh::dashboard(&win, &db);
            }
        }
    });

    register_program_status_callbacks(window, state);
}

fn register_program_status_callbacks(window: &MainWindow, state: AppState) {
    let window_weak = window.as_weak();
    let db = state.database.clone();
    let selected_client_id = state.selected_client_id.clone();
    window.on_pause_program(move |cp_id| {
        let db = db.borrow();
        let _ = db.pause_client_program(cp_id as i64);
        if let Some(win) = window_weak.upgrade() {
            refresh::dashboard(&win, &db);
            let client_id = *selected_client_id.borrow();
            if client_id >= 0 {
                refresh::client_programs(&win, &db, client_id);
            }
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let selected_client_id = state.selected_client_id.clone();
    window.on_resume_program(move |cp_id| {
        let db = db.borrow();
        let _ = db.resume_client_program(cp_id as i64);
        if let Some(win) = window_weak.upgrade() {
            refresh::dashboard(&win, &db);
            let client_id = *selected_client_id.borrow();
            if client_id >= 0 {
                refresh::client_programs(&win, &db, client_id);
            }
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let selected_client_id = state.selected_client_id.clone();
    window.on_complete_program(move |cp_id| {
        let db = db.borrow();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let _ = db.complete_client_program(cp_id as i64, &today);
        if let Some(win) = window_weak.upgrade() {
            refresh::dashboard(&win, &db);
            let client_id = *selected_client_id.borrow();
            if client_id >= 0 {
                refresh::client_programs(&win, &db, client_id);
            }
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let selected_client_id = state.selected_client_id.clone();
    window.on_cancel_program(move |cp_id| {
        let db = db.borrow();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let _ = db.cancel_client_program(cp_id as i64, &today);
        if let Some(win) = window_weak.upgrade() {
            refresh::dashboard(&win, &db);
            let client_id = *selected_client_id.borrow();
            if client_id >= 0 {
                refresh::client_programs(&win, &db, client_id);
            }
        }
    });
}

fn trainer_idx_to_id(db: &db::Db, trainer_idx: i32) -> Option<i64> {
    if trainer_idx <= 0 {
        return None;
    }
    let trainers = db.get_trainers().unwrap_or_default();
    trainers.get((trainer_idx - 1) as usize).map(|t| t.id)
}
