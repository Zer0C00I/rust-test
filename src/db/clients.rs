use rusqlite::{params, Result};

use super::Db;

#[derive(Debug, Clone)]
pub struct Client {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub email: String,
    pub notes: String,
    pub trainer_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ClientProgram {
    pub id: i64,
    pub client_id: i64,
    pub program_id: i64,
    pub status: String,
    pub start_date: String,
    pub end_date: String,
}

impl Db {
    pub fn add_client(
        &self,
        name: &str,
        phone: &str,
        email: &str,
        notes: &str,
        trainer_id: Option<i64>,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO clients (name, phone, email, notes, trainer_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, phone, email, notes, trainer_id],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_client(
        &self,
        id: i64,
        name: &str,
        phone: &str,
        email: &str,
        notes: &str,
        trainer_id: Option<i64>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE clients SET name=?1, phone=?2, email=?3, notes=?4, trainer_id=?5 WHERE id=?6",
            params![name, phone, email, notes, trainer_id, id],
        )?;
        Ok(())
    }

    pub fn delete_client(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM exercise_completions WHERE client_program_id IN \
             (SELECT id FROM client_programs WHERE client_id=?1)",
            params![id],
        )?;
        self.conn.execute(
            "DELETE FROM program_checkins WHERE client_program_id IN \
             (SELECT id FROM client_programs WHERE client_id=?1)",
            params![id],
        )?;
        self.conn.execute(
            "DELETE FROM client_programs WHERE client_id=?1",
            params![id],
        )?;
        self.conn
            .execute("DELETE FROM visits WHERE client_id=?1", params![id])?;
        self.conn
            .execute("DELETE FROM clients WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn get_clients(&self) -> Result<Vec<Client>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, phone, email, notes, trainer_id FROM clients ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Client {
                id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                email: row.get(3)?,
                notes: row.get(4)?,
                trainer_id: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn assign_program(
        &self,
        client_id: i64,
        program_id: i64,
        start_date: &str,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO client_programs (client_id, program_id, active, start_date, status, end_date) \
             VALUES (?1, ?2, 1, ?3, 'active', '')",
            params![client_id, program_id, start_date],
        )?;
        let cp_id = self.conn.last_insert_rowid();

        let duration: i64 = self.conn.query_row(
            "SELECT duration_weeks FROM training_programs WHERE id=?1",
            params![program_id],
            |row| row.get(0),
        )?;

        if let Ok(start) = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d") {
            for week in 0..duration {
                let date = start + chrono::Duration::weeks(week);
                self.conn.execute(
                    "INSERT INTO program_checkins (client_program_id, date, completed) \
                     VALUES (?1, ?2, 0)",
                    params![cp_id, date.format("%Y-%m-%d").to_string()],
                )?;
            }
        }

        Ok(cp_id)
    }

    pub fn get_active_client_programs(
        &self,
    ) -> Result<Vec<(ClientProgram, String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT cp.id, cp.client_id, cp.program_id, cp.status, cp.start_date, cp.end_date,
                    c.name, tp.name, COALESCE(t.name, 'None')
             FROM client_programs cp
             JOIN clients c ON c.id = cp.client_id
             JOIN training_programs tp ON tp.id = cp.program_id
             LEFT JOIN trainers t ON t.id = c.trainer_id
             WHERE cp.status IN ('active', 'paused')
             ORDER BY cp.start_date DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                ClientProgram {
                    id: row.get(0)?,
                    client_id: row.get(1)?,
                    program_id: row.get(2)?,
                    status: row.get(3)?,
                    start_date: row.get(4)?,
                    end_date: row.get(5)?,
                },
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
            ))
        })?;
        rows.collect()
    }

    pub fn get_client_programs_for_client(
        &self,
        client_id: i64,
    ) -> Result<Vec<(ClientProgram, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT cp.id, cp.client_id, cp.program_id, cp.status, cp.start_date, cp.end_date,
                    tp.name
             FROM client_programs cp
             JOIN training_programs tp ON tp.id = cp.program_id
             WHERE cp.client_id=?1
             ORDER BY cp.start_date DESC",
        )?;
        let rows = stmt.query_map(params![client_id], |row| {
            Ok((
                ClientProgram {
                    id: row.get(0)?,
                    client_id: row.get(1)?,
                    program_id: row.get(2)?,
                    status: row.get(3)?,
                    start_date: row.get(4)?,
                    end_date: row.get(5)?,
                },
                row.get::<_, String>(6)?,
            ))
        })?;
        rows.collect()
    }

    pub fn pause_client_program(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE client_programs SET status='paused' WHERE id=?1 AND status='active'",
            params![id],
        )?;
        Ok(())
    }

    pub fn resume_client_program(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE client_programs SET status='active' WHERE id=?1 AND status='paused'",
            params![id],
        )?;
        Ok(())
    }

    pub fn complete_client_program(&self, id: i64, end_date: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE client_programs SET status='completed', active=0, end_date=?2 \
             WHERE id=?1 AND status='active'",
            params![id, end_date],
        )?;
        Ok(())
    }

    pub fn cancel_client_program(&self, id: i64, end_date: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE client_programs SET status='cancelled', active=0, end_date=?2 \
             WHERE id=?1 AND status IN ('active','paused')",
            params![id, end_date],
        )?;
        Ok(())
    }
}
