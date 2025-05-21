use deadpool_postgres::{ Pool };
use tokio_postgres::NoTls;

pub type DbPool = Pool;

pub fn init_pool(cfg: &deadpool_postgres::Config) -> DbPool {
    cfg.create_pool(None, NoTls).expect("Postgres connection pool 생성 실패")
}
