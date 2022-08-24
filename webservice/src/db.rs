extern crate postgres;

use postgres::{Connection, TlsMode};


pub fn db_conn() -> Option<Connection> {
    let conn = Connection::connect("<postgres-server>", TlsMode::None).ok()?;
    Some(conn)
}