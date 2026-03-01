use rusqlite::{Result, params};

use super::Db;

#[derive(Debug, Clone)]
pub struct Visit {
    pub id: i64,
    pub client_id: i64,
    pub date: String,
    pub amount: f64,
    pub notes: String,
}

impl Db {
    pub fn add_visit(&self, client_id: i64, date: &str, amount: f64, notes: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO visits (client_id, date, amount, notes) VALUES (?1, ?2, ?3, ?4)",
            params![client_id, date, amount, notes],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_visit(
        &self,
        id: i64,
        client_id: i64,
        date: &str,
        amount: f64,
        notes: &str,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE visits SET client_id=?1, date=?2, amount=?3, notes=?4 WHERE id=?5",
            params![client_id, date, amount, notes, id],
        )?;
        Ok(())
    }

    pub fn delete_visit(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM visits WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn get_visits_for_month(&self, year: i32, month: i32) -> Result<Vec<(Visit, String)>> {
        let prefix = format!("{:04}-{:02}", year, month);
        let mut stmt = self.conn.prepare(
            "SELECT v.id, v.client_id, v.date, v.amount, v.notes, c.name
             FROM visits v
             JOIN clients c ON c.id = v.client_id
             WHERE v.date LIKE ?1
             ORDER BY v.date DESC, v.id DESC",
        )?;
        let rows = stmt.query_map(params![format!("{}%", prefix)], |row| {
            Ok((
                Visit {
                    id: row.get(0)?,
                    client_id: row.get(1)?,
                    date: row.get(2)?,
                    amount: row.get(3)?,
                    notes: row.get(4)?,
                },
                row.get::<_, String>(5)?,
            ))
        })?;
        rows.collect()
    }

    pub fn get_monthly_total(&self, year: i32, month: i32) -> Result<f64> {
        let prefix = format!("{:04}-{:02}", year, month);
        self.conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM visits WHERE date LIKE ?1",
            params![format!("{}%", prefix)],
            |row| row.get(0),
        )
    }
}
