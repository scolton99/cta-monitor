use mysql::Result;
use mysql::prelude::Queryable;

pub trait Record : Sized {
    fn all<T: Queryable>(conn: &mut T) -> Result<Vec<Self>>;
    fn save<T: Queryable>(&mut self, conn: &mut T) -> Result<()>;
    fn reload<T: Queryable>(&mut self, conn: &mut T) -> Result<()>;
    fn destroy_all<T: Queryable>(conn: &mut T) -> Result<()>;
    fn save_all<T: Queryable>(conn: &mut T, items: &mut [Self]) -> Result<()>;
}