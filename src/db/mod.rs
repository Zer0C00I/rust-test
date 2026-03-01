mod appointments;
mod checkins;
mod clients;
mod programs;
mod trainers;
mod visits;

use rusqlite::{Connection, Result};
use chrono;

pub use checkins::ProgramCheckIn;

pub struct Db {
    pub conn: Connection,
}

impl Db {
    pub fn open() -> Result<Self> {
        let path = db_path();
        backup_on_open(&path);
        let conn = Connection::open(&path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        let db = Db { conn };
        db.create_tables()?;
        db.run_migrations()?;
        Ok(db)
    }
}

fn backup_on_open(db_path: &std::path::Path) {
    if !db_path.exists() {
        return;
    }
    let backup_dir = db_path.parent().unwrap().join("backups");
    if std::fs::create_dir_all(&backup_dir).is_err() {
        return;
    }
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let dest = backup_dir.join(format!("crm_{}.db", today));
    if dest.exists() {
        return; // already backed up today
    }
    let _ = std::fs::copy(db_path, &dest);

    // keep last 7 daily backups
    if let Ok(mut entries) = std::fs::read_dir(&backup_dir) {
        let mut files: Vec<_> = entries
            .by_ref()
            .flatten()
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("crm_")
            })
            .collect();
        files.sort_by_key(|e| e.file_name());
        if files.len() > 7 {
            for old in &files[..files.len() - 7] {
                let _ = std::fs::remove_file(old.path());
            }
        }
    }
}

fn db_path() -> std::path::PathBuf {
    let data_dir = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME").unwrap_or_else(|| "/tmp".into());
            std::path::PathBuf::from(home).join(".local/share")
        })
        .join("slint-crm");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir.join("crm.db")
}

impl Db {
    fn run_migrations(&self) -> Result<()> {
        let has_status = self
            .conn
            .prepare("SELECT status FROM client_programs LIMIT 0")
            .is_ok();
        if !has_status {
            self.conn.execute_batch(
                "ALTER TABLE client_programs ADD COLUMN status TEXT NOT NULL DEFAULT 'active';
                 ALTER TABLE client_programs ADD COLUMN end_date TEXT NOT NULL DEFAULT '';
                 UPDATE client_programs SET status = 'active' WHERE active = 1;
                 UPDATE client_programs SET status = 'completed' WHERE active = 0;",
            )?;
        }
        let has_weight = self
            .conn
            .prepare("SELECT weight FROM exercises LIMIT 0")
            .is_ok();
        if !has_weight {
            self.conn.execute_batch(
                "ALTER TABLE exercises ADD COLUMN weight INTEGER NOT NULL DEFAULT 0;",
            )?;
        }
        let has_appt_status = self
            .conn
            .prepare("SELECT status FROM appointments LIMIT 0")
            .is_ok();
        if !has_appt_status {
            self.conn.execute_batch(
                "ALTER TABLE appointments ADD COLUMN status TEXT NOT NULL DEFAULT 'pending';
                 ALTER TABLE appointments ADD COLUMN amount REAL NOT NULL DEFAULT 0;",
            )?;
        }
        Ok(())
    }

    fn create_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS trainers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                phone TEXT NOT NULL DEFAULT '',
                specialization TEXT NOT NULL DEFAULT ''
            );
            CREATE TABLE IF NOT EXISTS clients (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                phone TEXT NOT NULL DEFAULT '',
                email TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                trainer_id INTEGER,
                FOREIGN KEY (trainer_id) REFERENCES trainers(id)
            );
            CREATE TABLE IF NOT EXISTS training_programs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                duration_weeks INTEGER NOT NULL DEFAULT 4
            );
            CREATE TABLE IF NOT EXISTS exercises (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                program_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                sets INTEGER NOT NULL DEFAULT 3,
                reps INTEGER NOT NULL DEFAULT 10,
                notes TEXT NOT NULL DEFAULT '',
                FOREIGN KEY (program_id) REFERENCES training_programs(id)
            );
            CREATE TABLE IF NOT EXISTS client_programs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                client_id INTEGER NOT NULL,
                program_id INTEGER NOT NULL,
                active INTEGER NOT NULL DEFAULT 1,
                start_date TEXT NOT NULL,
                FOREIGN KEY (client_id) REFERENCES clients(id),
                FOREIGN KEY (program_id) REFERENCES training_programs(id)
            );
            CREATE TABLE IF NOT EXISTS program_checkins (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                client_program_id INTEGER NOT NULL,
                date TEXT NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (client_program_id) REFERENCES client_programs(id)
            );
            CREATE TABLE IF NOT EXISTS exercise_completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                client_program_id INTEGER NOT NULL,
                exercise_id INTEGER NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (client_program_id) REFERENCES client_programs(id),
                FOREIGN KEY (exercise_id) REFERENCES exercises(id),
                UNIQUE(client_program_id, exercise_id)
            );
            CREATE TABLE IF NOT EXISTS visits (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                client_id INTEGER NOT NULL,
                date TEXT NOT NULL,
                amount REAL NOT NULL DEFAULT 0,
                notes TEXT NOT NULL DEFAULT '',
                FOREIGN KEY (client_id) REFERENCES clients(id)
            );
            CREATE TABLE IF NOT EXISTS exercise_checkin_completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                client_program_id INTEGER NOT NULL,
                exercise_id INTEGER NOT NULL,
                checkin_id INTEGER NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (client_program_id) REFERENCES client_programs(id),
                FOREIGN KEY (exercise_id) REFERENCES exercises(id),
                FOREIGN KEY (checkin_id) REFERENCES program_checkins(id),
                UNIQUE(client_program_id, exercise_id, checkin_id)
            );
            CREATE TABLE IF NOT EXISTS appointments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                client_id INTEGER NOT NULL,
                date TEXT NOT NULL,
                time TEXT NOT NULL,
                duration_minutes INTEGER NOT NULL DEFAULT 60,
                notes TEXT NOT NULL DEFAULT '',
                FOREIGN KEY (client_id) REFERENCES clients(id)
            );",
        )
    }
}
