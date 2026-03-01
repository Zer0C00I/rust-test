use std::rc::Rc;

use chrono::NaiveDate;

use crate::db;
use crate::{
    AppointmentData, CheckInData, ClientData, ClientProgramData, DashboardCardData,
    ExerciseCompletionData, ExerciseData, MainWindow, ProgramData, TrainerData, VisitData,
    WeekDayData,
};

pub fn trainers(window: &MainWindow, db: &db::Db) {
    let rows = db.get_trainers().unwrap_or_default();
    let model: Vec<TrainerData> = rows
        .iter()
        .map(|t| TrainerData {
            id: t.id as i32,
            name: t.name.clone().into(),
            phone: t.phone.clone().into(),
            specialization: t.specialization.clone().into(),
        })
        .collect();
    window.set_trainers(Rc::new(slint::VecModel::from(model)).into());
}

pub fn programs(window: &MainWindow, db: &db::Db) {
    let rows = db.get_programs().unwrap_or_default();
    let model: Vec<ProgramData> = rows
        .iter()
        .map(|p| ProgramData {
            id: p.id as i32,
            name: p.name.clone().into(),
            description: p.description.clone().into(),
            duration_weeks: p.duration_weeks as i32,
        })
        .collect();
    window.set_programs(Rc::new(slint::VecModel::from(model)).into());
}

pub fn clients(window: &MainWindow, db: &db::Db) {
    let client_rows = db.get_clients().unwrap_or_default();
    let trainer_rows = db.get_trainers().unwrap_or_default();
    let model: Vec<ClientData> = client_rows
        .iter()
        .map(|c| {
            let trainer_pos = c
                .trainer_id
                .and_then(|tid| trainer_rows.iter().position(|t| t.id == tid));
            let trainer_name = trainer_pos
                .map(|pos| trainer_rows[pos].name.clone())
                .unwrap_or_default();
            let trainer_idx = trainer_pos.map(|pos| (pos + 1) as i32).unwrap_or(0);
            ClientData {
                id: c.id as i32,
                name: c.name.clone().into(),
                phone: c.phone.clone().into(),
                email: c.email.clone().into(),
                notes: c.notes.clone().into(),
                trainer_id: c.trainer_id.unwrap_or(-1) as i32,
                trainer_idx,
                trainer_name: trainer_name.into(),
            }
        })
        .collect();
    window.set_clients(Rc::new(slint::VecModel::from(model)).into());
}

pub fn dashboard(window: &MainWindow, db: &db::Db) {
    let rows = db.get_active_client_programs().unwrap_or_default();
    let model: Vec<DashboardCardData> = rows
        .iter()
        .map(|(cp, client_name, program_name, trainer_name)| DashboardCardData {
            id: cp.id as i32,
            client_name: client_name.clone().into(),
            program_name: program_name.clone().into(),
            trainer_name: trainer_name.clone().into(),
            start_date: cp.start_date.clone().into(),
            status: cp.status.clone().into(),
        })
        .collect();
    window.set_active_programs(Rc::new(slint::VecModel::from(model)).into());
}

pub fn exercises(window: &MainWindow, db: &db::Db, program_id: i64) {
    let rows = db.get_exercises_for_program(program_id).unwrap_or_default();
    let model: Vec<ExerciseData> = rows
        .iter()
        .map(|e| ExerciseData {
            id: e.id as i32,
            name: e.name.clone().into(),
            sets: e.sets as i32,
            reps: e.reps as i32,
            weight: e.weight as i32,
            notes: e.notes.clone().into(),
        })
        .collect();
    window.set_exercises(Rc::new(slint::VecModel::from(model)).into());
}

pub fn client_programs(window: &MainWindow, db: &db::Db, client_id: i64) {
    let rows = db
        .get_client_programs_for_client(client_id)
        .unwrap_or_default();
    let model: Vec<ClientProgramData> = rows
        .iter()
        .map(|(cp, program_name)| ClientProgramData {
            id: cp.id as i32,
            program_name: program_name.clone().into(),
            status: cp.status.clone().into(),
            start_date: cp.start_date.clone().into(),
        })
        .collect();
    window.set_selected_client_programs(Rc::new(slint::VecModel::from(model)).into());
}

pub fn checkins(window: &MainWindow, rows: &[db::ProgramCheckIn]) {
    let model: Vec<CheckInData> = rows
        .iter()
        .map(|ci| CheckInData {
            id: ci.id as i32,
            date: ci.date.clone().into(),
            completed: ci.completed,
        })
        .collect();
    window.set_checkins(Rc::new(slint::VecModel::from(model)).into());
}

pub fn exercise_completions(window: &MainWindow, db: &db::Db, client_program_id: i64) {
    let rows = db
        .get_exercise_completions(client_program_id)
        .unwrap_or_default();
    let model: Vec<ExerciseCompletionData> = rows
        .iter()
        .map(|ec| ExerciseCompletionData {
            id: ec.id as i32,
            exercise_id: ec.exercise_id as i32,
            name: ec.name.clone().into(),
            sets: ec.sets as i32,
            reps: ec.reps as i32,
            weight: ec.weight as i32,
            notes: ec.notes.clone().into(),
            completed: ec.completed,
        })
        .collect();
    window.set_exercise_completions(Rc::new(slint::VecModel::from(model)).into());
}

pub fn exercise_week_completions(window: &MainWindow, db: &db::Db, client_program_id: i64) {
    let rows = db
        .get_exercise_week_completions(client_program_id)
        .unwrap_or_default();
    let model: Vec<bool> = rows.iter().map(|(_, _, done)| *done).collect();
    window.set_exercise_week_done(Rc::new(slint::VecModel::from(model)).into());
}

pub fn visits(window: &MainWindow, db: &db::Db, year: i32, month: i32) {
    let visit_rows = db.get_visits_for_month(year, month).unwrap_or_default();
    let client_rows = db.get_clients().unwrap_or_default();
    let model: Vec<VisitData> = visit_rows
        .iter()
        .map(|(v, client_name)| {
            let client_idx = client_rows
                .iter()
                .position(|c| c.id == v.client_id)
                .unwrap_or(0);
            VisitData {
                id: v.id as i32,
                client_id: v.client_id as i32,
                client_idx: client_idx as i32,
                client_name: client_name.clone().into(),
                date: v.date.clone().into(),
                amount: v.amount as f32,
                notes: v.notes.clone().into(),
            }
        })
        .collect();
    window.set_visits(Rc::new(slint::VecModel::from(model)).into());
    let total = db.get_monthly_total(year, month).unwrap_or(0.0);
    window.set_monthly_total(total as f32);
}

pub fn schedule(window: &MainWindow, db: &db::Db, week_start: &str) {
    let start_date = NaiveDate::parse_from_str(week_start, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::Local::now().date_naive());

    let week_end = start_date + chrono::Duration::days(6);
    let week_end_str = week_end.format("%Y-%m-%d").to_string();

    let today = chrono::Local::now().date_naive();

    let day_names_en = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let day_names_ua = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Нд"];

    // Build week-days
    let week_days: Vec<WeekDayData> = (0..7)
        .map(|i| {
            let d = start_date + chrono::Duration::days(i);
            let day_idx = i as i32;
            let is_today = d == today;
            // Label: "Mon\n23" style — just "Mon 23" as a single string (newline not supported in slint Text this way)
            let label_en = format!("{}\n{}", day_names_en[i as usize], d.format("%d"));
            let label_ua = format!("{}\n{}", day_names_ua[i as usize], d.format("%d"));
            let _ = label_ua; // used by Slint translation
            WeekDayData {
                label: label_en.into(),
                date: d.format("%Y-%m-%d").to_string().into(),
                is_today: is_today,
                day_index: day_idx,
            }
        })
        .collect();

    // Week label: "Feb 23 – Mar 1, 2026"
    let week_label = format!(
        "{} – {}",
        start_date.format("%b %-d"),
        week_end.format("%b %-d, %Y")
    );

    // Hour labels 06:00..21:00
    let hour_labels: Vec<slint::SharedString> = (6..=21)
        .map(|h| format!("{:02}:00", h).into())
        .collect();

    // Fetch appointments
    let client_rows = db.get_clients().unwrap_or_default();
    let appts = db
        .get_appointments_for_week(week_start, &week_end_str)
        .unwrap_or_default();

    let appointments: Vec<AppointmentData> = appts
        .iter()
        .map(|(a, client_name)| {
            let appt_date = NaiveDate::parse_from_str(&a.date, "%Y-%m-%d")
                .unwrap_or(start_date);
            let day_index = (appt_date - start_date).num_days() as i32;

            let (hour, minute) = parse_time(&a.time);

            let client_idx = client_rows
                .iter()
                .position(|c| c.id == a.client_id)
                .unwrap_or(0) as i32;

            AppointmentData {
                id: a.id as i32,
                client_id: a.client_id as i32,
                client_idx,
                client_name: client_name.clone().into(),
                date: a.date.clone().into(),
                time: a.time.clone().into(),
                hour,
                minute,
                day_index,
                duration_minutes: a.duration_minutes as i32,
                notes: a.notes.clone().into(),
                status: a.status.clone().into(),
                amount: a.amount as f32,
            }
        })
        .collect();

    window.set_week_days(Rc::new(slint::VecModel::from(week_days)).into());
    window.set_appointments(Rc::new(slint::VecModel::from(appointments)).into());
    window.set_hour_labels(Rc::new(slint::VecModel::from(hour_labels)).into());
    window.set_week_label(week_label.into());
}

fn parse_time(time: &str) -> (i32, i32) {
    let parts: Vec<&str> = time.splitn(2, ':').collect();
    let hour = parts.first().and_then(|s| s.parse().ok()).unwrap_or(6);
    let minute = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    (hour, minute)
}
