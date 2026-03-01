use slint::ComponentHandle;

use crate::{AppState, MainWindow, refresh};

pub(super) fn register(window: &MainWindow, state: AppState) {
    let window_weak = window.as_weak();
    let db = state.database.clone();
    let week_start = state.current_week_start.clone();
    window.on_prev_week(move || {
        let ws = {
            let s = week_start.borrow();
            prev_week_start(&s)
        };
        *week_start.borrow_mut() = ws.clone();
        let db = db.borrow();
        if let Some(win) = window_weak.upgrade() {
            refresh::schedule(&win, &db, &ws);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let week_start = state.current_week_start.clone();
    window.on_next_week(move || {
        let ws = {
            let s = week_start.borrow();
            next_week_start(&s)
        };
        *week_start.borrow_mut() = ws.clone();
        let db = db.borrow();
        if let Some(win) = window_weak.upgrade() {
            refresh::schedule(&win, &db, &ws);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let week_start = state.current_week_start.clone();
    window.on_add_appointment(move |client_id, date, time, duration_minutes, notes| {
        let db = db.borrow();
        let _ = db.add_appointment(
            client_id as i64,
            &date,
            &time,
            duration_minutes as i64,
            &notes,
        );
        let ws = week_start.borrow().clone();
        if let Some(win) = window_weak.upgrade() {
            refresh::schedule(&win, &db, &ws);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let week_start = state.current_week_start.clone();
    window.on_update_appointment(move |id, client_id, date, time, duration_minutes, notes| {
        let db = db.borrow();
        let _ = db.update_appointment(
            id as i64,
            client_id as i64,
            &date,
            &time,
            duration_minutes as i64,
            &notes,
        );
        let ws = week_start.borrow().clone();
        if let Some(win) = window_weak.upgrade() {
            refresh::schedule(&win, &db, &ws);
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let week_start = state.current_week_start.clone();
    window.on_delete_appointment(move |id| {
        let db = db.borrow();
        let _ = db.delete_appointment(id as i64);
        let ws = week_start.borrow().clone();
        if let Some(win) = window_weak.upgrade() {
            refresh::schedule(&win, &db, &ws);
        }
    });

    // mark_attended(id, client_id, date, amount, visit_note)
    // Marks the appointment as attended; if amount > 0 also adds a visit to finances.
    let window_weak = window.as_weak();
    let db = state.database.clone();
    let week_start = state.current_week_start.clone();
    let current_year = state.current_year.clone();
    let current_month = state.current_month.clone();
    window.on_mark_attended(move |id, client_id, date, amount, visit_note| {
        let db = db.borrow();
        let _ = db.mark_attended(id as i64, amount as f64);
        if amount > 0.0 {
            let _ = db.add_visit(client_id as i64, &date, amount as f64, &visit_note);
        }
        let ws = week_start.borrow().clone();
        if let Some(win) = window_weak.upgrade() {
            refresh::schedule(&win, &db, &ws);
            refresh::visits(&win, &db, *current_year.borrow(), *current_month.borrow());
        }
    });

    let window_weak = window.as_weak();
    let db = state.database.clone();
    let week_start = state.current_week_start.clone();
    window.on_mark_no_show(move |id| {
        let db = db.borrow();
        let _ = db.mark_no_show(id as i64);
        let ws = week_start.borrow().clone();
        if let Some(win) = window_weak.upgrade() {
            refresh::schedule(&win, &db, &ws);
        }
    });
}

fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

fn prev_week_start(current: &str) -> String {
    parse_date(current)
        .map(|d| {
            (d - chrono::Duration::days(7))
                .format("%Y-%m-%d")
                .to_string()
        })
        .unwrap_or_else(|| current.to_string())
}

fn next_week_start(current: &str) -> String {
    parse_date(current)
        .map(|d| {
            (d + chrono::Duration::days(7))
                .format("%Y-%m-%d")
                .to_string()
        })
        .unwrap_or_else(|| current.to_string())
}
