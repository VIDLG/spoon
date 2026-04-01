use crate::Result;
use crate::control_plane::sqlite::ControlPlaneDb;
use crate::layout::RuntimeLayout;
use rusqlite::params;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DoctorIssueRecord {
    pub severity: String,
    pub category: String,
    pub description: String,
    pub package: Option<String>,
    pub bucket: Option<String>,
    pub resolved: bool,
}

pub async fn sync_failed_lifecycle_issues(layout: &RuntimeLayout) -> Result<()> {
    let db = ControlPlaneDb::open_for_layout(layout).await?;
    db.call_write(move |conn| {
        conn.execute(
            "DELETE FROM doctor_issues WHERE category = 'failed_lifecycle'",
            [],
        )?;

        let mut stmt = conn.prepare(
            "SELECT operation_type, package, bucket, details
             FROM operation_journal
             WHERE status = 'failed'
             ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })?;

        for row in rows {
            let (operation_type, package, bucket, details) = row?;
            let package_label = package.as_deref().unwrap_or("<unknown>");
            let mut description = format!(
                "failed {operation_type} operation for package '{package_label}'"
            );
            if let Some(details) = details
                && !details.trim().is_empty()
            {
                description.push_str(": ");
                description.push_str(details.trim());
            }
            conn.execute(
                "INSERT INTO doctor_issues (severity, category, description, package, bucket, resolved)
                 VALUES ('error', 'failed_lifecycle', ?1, ?2, ?3, 0)",
                params![description, package, bucket],
            )?;
        }
        Ok(())
    })
    .await
}

pub async fn list_doctor_issues(layout: &RuntimeLayout) -> Result<Vec<DoctorIssueRecord>> {
    let db = ControlPlaneDb::open_for_layout(layout).await?;
    db.call(|conn| {
        let mut stmt = conn.prepare(
            "SELECT severity, category, description, package, bucket, resolved
             FROM doctor_issues ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DoctorIssueRecord {
                severity: row.get(0)?,
                category: row.get(1)?,
                description: row.get(2)?,
                package: row.get(3)?,
                bucket: row.get(4)?,
                resolved: row.get::<_, i64>(5)? != 0,
            })
        })?;
        Ok(rows.filter_map(|row| row.ok()).collect::<Vec<_>>())
    })
    .await
}
