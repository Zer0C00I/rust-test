use rusqlite::{params, Result};

use super::Db;

#[derive(Debug, Clone)]
pub struct Appointment {
    pub id: i64,
    pub client_id: i64,
    pub date: String,
    pub time: String,
    pub duration_minutes: i64,
    pub notes: String,
    pub status: String,   // "pending" | "attended" | "no-show"
    pub amount: f64,
}

impl Db {
    pub fn add_appointment(
        &self,
        client_id: i64,
        date: &str,
        time: &str,
        duration_minutes: i64,
        notes: &str,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO appointments (client_id, date, time, duration_minutes, notes, status, amount) \
             VALUES (?1, ?2, ?3, ?4, ?5, 'pending', 0)",
            params![client_id, date, time, duration_minutes, notes],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_appointment(
        &self,
        id: i64,
        client_id: i64,
        date: &str,
        time: &str,
        duration_minutes: i64,
        notes: &str,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE appointments SET client_id=?1, date=?2, time=?3, duration_minutes=?4, notes=?5 \
             WHERE id=?6",
            params![client_id, date, time, duration_minutes, notes, id],
        )?;
        Ok(())
    }

    pub fn mark_attended(&self, id: i64, amount: f64) -> Result<()> {
        self.conn.execute(
            "UPDATE appointments SET status='attended', amount=?1 WHERE id=?2",
            params![amount, id],
        )?;
        Ok(())
    }

    pub fn mark_no_show(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE appointments SET status='no-show' WHERE id=?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn delete_appointment(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM appointments WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn get_appointments_for_week(
        &self,
        start: &str,
        end: &str,
    ) -> Result<Vec<(Appointment, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.id, a.client_id, a.date, a.time, a.duration_minutes, a.notes,
                    a.status, a.amount, c.name
             FROM appointments a
             JOIN clients c ON c.id = a.client_id
             WHERE a.date >= ?1 AND a.date <= ?2
             ORDER BY a.date ASC, a.time ASC",
        )?;
        let rows = stmt.query_map(params![start, end], |row| {
            Ok((
                Appointment {
                    id: row.get(0)?,
                    client_id: row.get(1)?,
                    date: row.get(2)?,
                    time: row.get(3)?,
                    duration_minutes: row.get(4)?,
                    notes: row.get(5)?,
                    status: row.get(6)?,
                    amount: row.get(7)?,
                },
                row.get::<_, String>(8)?,
            ))
        })?;
        rows.collect()
    }
}
