use crate::{AppState, MainWindow};

pub(super) fn register(window: &MainWindow, state: AppState) {
    let db = state.database.clone();
    window.on_save_pdf(move |ukrainian, cp_id| {
        let db = db.borrow();
        if let Err(e) = crate::print::save_pdf(&db, ukrainian, cp_id as i64) {
            eprintln!("PDF save error: {e}");
        }
    });

    let db = state.database.clone();
    window.on_print_report(move |ukrainian, cp_id| {
        let db = db.borrow();
        if let Err(e) = crate::print::print_pdf(&db, ukrainian, cp_id as i64) {
            eprintln!("Print error: {e}");
        }
    });
}
