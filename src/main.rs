mod db;
mod handlers;
mod print;
mod refresh;

use std::cell::RefCell;
use std::rc::Rc;

use chrono::Datelike;

slint::include_modules!();

#[derive(Clone)]
pub struct AppState {
    pub database: Rc<RefCell<db::Db>>,
    pub current_year: Rc<RefCell<i32>>,
    pub current_month: Rc<RefCell<i32>>,
    pub selected_program_id: Rc<RefCell<i64>>,
    pub selected_client_id: Rc<RefCell<i64>>,
    pub expanded_program_id: Rc<RefCell<i64>>,
    pub current_week_start: Rc<RefCell<String>>,
}

fn main() {
    
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");
    let _rt_guard = rt.enter();

    let window = MainWindow::new().unwrap();
    let now = chrono::Local::now();

    // Compute the Monday of the current week (ISO week Mon=0)
    let week_start = {
        use chrono::Datelike;
        let weekday_from_mon = now.weekday().num_days_from_monday() as i64;
        let monday = now.date_naive() - chrono::Duration::days(weekday_from_mon);
        monday.format("%Y-%m-%d").to_string()
    };

    let state = AppState {
        database: Rc::new(RefCell::new(
            db::Db::open().expect("Failed to open database"),
        )),
        current_year: Rc::new(RefCell::new(now.year())),
        current_month: Rc::new(RefCell::new(now.month() as i32)),
        selected_program_id: Rc::new(RefCell::new(-1)),
        selected_client_id: Rc::new(RefCell::new(-1)),
        expanded_program_id: Rc::new(RefCell::new(-1)),
        current_week_start: Rc::new(RefCell::new(week_start.clone())),
    };

    {
        let db = state.database.borrow();
        let year = *state.current_year.borrow();
        let month = *state.current_month.borrow();
        refresh::trainers(&window, &db);
        refresh::programs(&window, &db);
        refresh::clients(&window, &db);
        refresh::dashboard(&window, &db);
        window.set_current_year(year);
        window.set_current_month(month);
        refresh::visits(&window, &db, year, month);
        refresh::schedule(&window, &db, &week_start);
    }

    handlers::register_all(&window, state);
    window.run().unwrap();
}
