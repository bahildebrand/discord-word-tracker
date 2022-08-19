use rand::Rng;
use rocksdb::{Options, DB};
use std::sync::Arc;

use crate::counter_db::CounterDb;

pub struct TestDbWrapper {
    counter_db: Option<Arc<CounterDb>>,
    path: String,
}

impl TestDbWrapper {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let rng_num: u64 = rng.gen();
        let path = format!("rocksdb-test-{}", rng_num);

        Self {
            counter_db: Some(Arc::new(CounterDb::new(path.clone()))),
            path,
        }
    }

    pub fn get_db(&self) -> Arc<CounterDb> {
        self.counter_db.as_ref().unwrap().clone()
    }
}

impl Drop for TestDbWrapper {
    fn drop(&mut self) {
        drop(self.counter_db.take().unwrap());
        DB::destroy(&Options::default(), self.path.clone()).unwrap();
    }
}
