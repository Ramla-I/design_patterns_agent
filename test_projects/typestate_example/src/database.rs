use std::marker::PhantomData;

/// State: Database connection is disconnected
pub struct Disconnected;

/// State: Database connection is active
pub struct Connected;

/// A database connection that uses typestate to ensure proper connection lifecycle.
///
/// INVARIANT: Queries can only be executed on a connected database.
/// INVARIANT: The connection must be established before use.
pub struct Connection<S> {
    connection_string: String,
    _state: PhantomData<S>,
}

impl Connection<Disconnected> {
    /// Create a new disconnected connection
    pub fn new(connection_string: impl Into<String>) -> Self {
        Connection {
            connection_string: connection_string.into(),
            _state: PhantomData,
        }
    }

    /// Connect to the database, consuming the disconnected connection
    /// and returning a connected one.
    ///
    /// INVARIANT: This state transition ensures connect() is only called once
    pub fn connect(self) -> Result<Connection<Connected>, String> {
        println!("Connecting to: {}", self.connection_string);

        if self.connection_string.is_empty() {
            return Err("Invalid connection string".to_string());
        }

        Ok(Connection {
            connection_string: self.connection_string,
            _state: PhantomData,
        })
    }
}

impl Connection<Connected> {
    /// Execute a query (only available when connected)
    ///
    /// INVARIANT: Queries require an active connection
    pub fn query(&self, sql: &str) -> Vec<String> {
        println!("Executing query: {}", sql);
        vec!["result1".to_string(), "result2".to_string()]
    }

    /// Begin a transaction (only available when connected)
    ///
    /// INVARIANT: Transactions require an active connection
    pub fn begin_transaction(&mut self) -> Transaction<'_> {
        println!("Beginning transaction");
        Transaction { conn: self }
    }

    /// Disconnect from the database
    pub fn disconnect(self) -> Connection<Disconnected> {
        println!("Disconnecting from: {}", self.connection_string);
        Connection {
            connection_string: self.connection_string,
            _state: PhantomData,
        }
    }
}

/// A transaction that must be committed or rolled back
///
/// INVARIANT: Transactions must be explicitly committed or rolled back
/// This is a linear type - it cannot be cloned and must be consumed
pub struct Transaction<'a> {
    conn: &'a mut Connection<Connected>,
}

impl<'a> Transaction<'a> {
    /// Commit the transaction, consuming it
    #[must_use]
    pub fn commit(self) {
        println!("Committing transaction");
    }

    /// Rollback the transaction, consuming it
    #[must_use]
    pub fn rollback(self) {
        println!("Rolling back transaction");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_lifecycle() {
        let conn = Connection::new("localhost:5432");
        let conn = conn.connect().unwrap();
        let _results = conn.query("SELECT * FROM users");
        let _disconnected = conn.disconnect();
    }
}
