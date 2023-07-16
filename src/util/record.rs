use mysql::{Result, PooledConn, Conn, Transaction};

pub enum DB<'a> {
    Pooled(PooledConn),
    Standard(Conn),
    Tx(Transaction<'a>)
}

pub trait Record : Sized {
    fn all(conn: &mut DB) -> Result<Vec<Self>>;
    fn save(&mut self, conn: &mut DB) -> Result<()>;
    fn reload(&mut self, conn: &mut DB) -> Result<()>;
    fn destroy_all(conn: &mut DB) -> Result<()>;
    fn save_all(conn: &mut DB, items: &mut [Self]) -> Result<()>;
}