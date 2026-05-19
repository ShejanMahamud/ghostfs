use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

use crate::StoredPackage;

/// SQLite-backed metadata database for the global package store.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (or create) the database at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS packages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                hash TEXT NOT NULL,
                size INTEGER NOT NULL,
                installed_at TEXT NOT NULL,
                UNIQUE(name, version)
            );
            CREATE INDEX IF NOT EXISTS idx_packages_name ON packages(name);
            CREATE INDEX IF NOT EXISTS idx_packages_hash ON packages(hash);",
        )?;
        Ok(())
    }

    pub fn insert_package(&self, pkg: &StoredPackage) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO packages (name, version, hash, size, installed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![pkg.name, pkg.version, pkg.hash, pkg.size, pkg.installed_at],
        )?;
        Ok(())
    }

    pub fn has_package(&self, name: &str, version: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM packages WHERE name = ?1 AND version = ?2",
            rusqlite::params![name, version],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn get_package(&self, name: &str, version: &str) -> Result<Option<StoredPackage>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, version, hash, size, installed_at
             FROM packages WHERE name = ?1 AND version = ?2",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![name, version], |row| {
            Ok(StoredPackage {
                name: row.get(0)?,
                version: row.get(1)?,
                hash: row.get(2)?,
                size: row.get(3)?,
                installed_at: row.get(4)?,
            })
        })?;
        match rows.next() {
            Some(pkg) => Ok(Some(pkg?)),
            None => Ok(None),
        }
    }

    pub fn list_packages(&self) -> Result<Vec<StoredPackage>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, version, hash, size, installed_at
             FROM packages ORDER BY name, version",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(StoredPackage {
                name: row.get(0)?,
                version: row.get(1)?,
                hash: row.get(2)?,
                size: row.get(3)?,
                installed_at: row.get(4)?,
            })
        })?;
        let mut packages = Vec::new();
        for row in rows {
            packages.push(row?);
        }
        Ok(packages)
    }

    pub fn total_size(&self) -> Result<u64> {
        let size: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(size), 0) FROM packages",
            [],
            |row| row.get(0),
        )?;
        Ok(size as u64)
    }

    pub fn package_count(&self) -> Result<u64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM packages",
            [],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    pub fn clear_packages(&self) -> Result<()> {
        self.conn.execute("DELETE FROM packages", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_lifecycle() {
        // Open an in-memory database
        let db = Database::open(Path::new(":memory:")).unwrap();
        
        // Assert initial state is empty
        assert_eq!(db.package_count().unwrap(), 0);
        assert_eq!(db.total_size().unwrap(), 0);
        assert_eq!(db.list_packages().unwrap().len(), 0);

        // Define a test package
        let pkg = StoredPackage {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            hash: "abcdef0123456789".to_string(),
            size: 1024,
            installed_at: "2026-05-19T22:00:00Z".to_string(),
        };

        // Insert package
        db.insert_package(&pkg).unwrap();

        // Verify package exists
        assert!(db.has_package("test-package", "1.0.0").unwrap());
        assert!(!db.has_package("test-package", "2.0.0").unwrap());

        // Fetch package
        let fetched = db.get_package("test-package", "1.0.0").unwrap().unwrap();
        assert_eq!(fetched.name, "test-package");
        assert_eq!(fetched.version, "1.0.0");
        assert_eq!(fetched.hash, "abcdef0123456789");
        assert_eq!(fetched.size, 1024);

        // Verify list, count, and size
        assert_eq!(db.package_count().unwrap(), 1);
        assert_eq!(db.total_size().unwrap(), 1024);
        assert_eq!(db.list_packages().unwrap().len(), 1);

        // Clear packages
        db.clear_packages().unwrap();
        assert_eq!(db.package_count().unwrap(), 0);
        assert_eq!(db.total_size().unwrap(), 0);
    }
}
