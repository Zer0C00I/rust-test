use slint::ComponentHandle;

use crate::{refresh, AppState, MainWindow};

pub(super) fn register(window: &MainWindow, state: AppState) {
    let window_weak = window.as_weak();
    let db = state.database.clone();
    window.on_add_trainer(move |name, phone, spec| {
        let db = db.borrow();
        let _ = db.add_trainer(&name, &phone, &spec);
        if let Some(win) = window_weak.upgrade() {
            refresh::trainers(&win, &db);
            refresh::clients(&win, &db);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    window.on_update_trainer(move |id, name, phone, spec| {
        let db = db.borrow();
        let _ = db.update_trainer(id as i64, &name, &phone, &spec);
        if let Some(win) = window_weak.upgrade() {
            refresh::trainers(&win, &db);
            refresh::clients(&win, &db);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    window.on_delete_trainer(move |id| {
        let db = db.borrow();
        let _ = db.delete_trainer(id as i64);
        if let Some(win) = window_weak.upgrade() {
            refresh::trainers(&win, &db);
            refresh::clients(&win, &db);
        }
    });
}
