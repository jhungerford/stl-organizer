use refinery::embed_migrations;
use rusqlite::{Connection, OpenFlags};

// The `db` module wires the refinery migrations stored in `src-tauri/sql_migrations` into sqlite.
embed_migrations!("./sql_migrations");

/// DbError is a unified error type for database errors.
#[derive(Debug, Clone)]
pub struct DbError {
    pub message: String
}

impl From<rusqlite::Error> for DbError {
    fn from(err: rusqlite::Error) -> Self {
        DbError { message: err.to_string() }
    }
}

impl From<refinery::Error> for DbError {
    fn from(err: refinery::Error) -> Self {
        DbError { message: err.to_string() }
    }
}

/// `ConnectionManager` is a wrapper around a sqlite database, allowing callers to get a connection
/// or migrate the schema.
pub trait ConnectionManager: Send + Sync {
    /// Returns a connection to the database.  `Connection` automatically closes the connection
    /// when the variable is dropped, so callers don't need to manually close the connection.
    fn get_connection(&self) -> rusqlite::Result<Connection>;

    /// Migrates the schema by running the migrations in `src-tauri/sql_migrations`.
    fn migrate(&self) -> Result<(), DbError> {
        let mut conn = self.get_connection()?;
        migrations::runner().run(&mut conn)?;

        Ok(())
    }
}

/// `InMemoryConnectionManager` runs an in-memory SQLite DB that will run in the process's memory
/// for as long as the connection manager stays in scope.  Useful for testing.
pub struct InMemoryConnectionManager {
    /// In-memory DB stays alive as long as one connection is open.  Keep a connection
    /// for the lifetime of the InMemoryConnectionManager - the DB will be cleared once
    /// the connection manager is dropped.
    #[allow(dead_code)]
    conn: Connection,
    name: String,
}

impl InMemoryConnectionManager {
    /// Constructs a new `InMemoryConnectionManager` with the given unique name.
    /// If multiple databases need to exist (e.g. one per test), name distinguishes them.
    #[allow(dead_code)]
    pub fn new(name: &str) -> Result<InMemoryConnectionManager, DbError> {
        // rusqlite provides `Connection::open_in_memory`, but in-memory databases are allowed to
        // use shared cache when opened with a uri filename (file::memory: instead of :memory:)
        // and the open shared cache flag.  See https://sqlite.org/inmemorydb.html
        let conn = InMemoryConnectionManager::open_connection(name)?;

        Ok(InMemoryConnectionManager {
            conn,
            name: name.to_string()
        })
    }

    /// Opens a connection to the shared in-memory database.
    pub fn open_connection(name: &str) -> rusqlite::Result<Connection> {
        let flags = OpenFlags::default() | OpenFlags::SQLITE_OPEN_SHARED_CACHE;
        let path = format!("file:{}?mode=memory&cache=shared", name);

        Connection::open_with_flags(path, flags)
    }
}

/// `InMemoryConnectionManager`'s parent connection to the database isn't safe to use in multiple
/// threads, but it's open to give the DB the same lifetime as the connection manager.
/// Threads call `conn_manager.get_connection()` to get a thread-safe connection to the DB.
unsafe impl Sync for InMemoryConnectionManager {}

impl ConnectionManager for InMemoryConnectionManager {
    fn get_connection(&self) -> rusqlite::Result<Connection> {
        return InMemoryConnectionManager::open_connection(&self.name)
    }
}

/// `FileConnectionManager` is a persistent SQLite database stored in a file on disk.
pub struct FileConnectionManager {
    file: String
}

impl FileConnectionManager {
    #[allow(dead_code)]
    pub fn new(file: String) -> FileConnectionManager {
        FileConnectionManager { file }
    }
}

impl ConnectionManager for FileConnectionManager {
    fn get_connection(&self) -> rusqlite::Result<Connection> {
        // TODO: DB shows up as locked when stl-organizer is run on windows with the DB on the wsl
        //       filesystem.  SQLite allows specifying a 'vfs' virtual file system object -
        //       maybe that could help?  http://www.sqlite.org/c3ref/open.html
        return Connection::open(&self.file);
    }
}
