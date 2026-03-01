use slint::ComponentHandle;

use crate::{AppState, MainWindow, refresh};

pub(super) fn register(window: &MainWindow, state: AppState) {
    let window_weak = window.as_weak();
    let db = state.database.clone();
    let current_year = state.current_year.clone();
    let current_month = state.current_month.clone();
    window.on_add_visit(move |client_id, date, amount, notes| {
        let db = db.borrow();
        let _ = db.add_visit(client_id as i64, &date, amount as f64, &notes);
        if let Some(win) = window_weak.upgrade() {
            refresh::visits(&win, &db, *current_year.borrow(), *current_month.borrow());
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let current_year = state.current_year.clone();
    let current_month = state.current_month.clone();
    window.on_update_visit(move |id, client_id, date, amount, notes| {
        let db = db.borrow();
        let _ = db.update_visit(id as i64, client_id as i64, &date, amount as f64, &notes);
        if let Some(win) = window_weak.upgrade() {
            refresh::visits(&win, &db, *current_year.borrow(), *current_month.borrow());
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let current_year = state.current_year.clone();
    let current_month = state.current_month.clone();
    window.on_delete_visit(move |id| {
        let db = db.borrow();
        let _ = db.delete_visit(id as i64);
        if let Some(win) = window_weak.upgrade() {
            refresh::visits(&win, &db, *current_year.borrow(), *current_month.borrow());
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let current_year = state.current_year.clone();
    let current_month = state.current_month.clone();
    window.on_change_month(move |year, month| {
        *current_year.borrow_mut() = year;
        *current_month.borrow_mut() = month;
        let db = db.borrow();
        if let Some(win) = window_weak.upgrade() {
            win.set_current_year(year);
            win.set_current_month(month);
            refresh::visits(&win, &db, year, month);
        }
    });
}
