use rocksdb::{Direction, IteratorMode, MergeOperands, Options, DB};
use tracing::debug;

pub struct CounterDb {
    db_client: DB,
}

impl CounterDb {
    pub fn new(db_path: String) -> Self {
        let mut opts = Options::default();

        opts.create_if_missing(true);
        opts.set_merge_operator_associative("inc operator", increment_merge);
        let db_client = DB::open(&opts, db_path).unwrap();

        Self { db_client }
    }

    pub fn inc_key(&self, key: &str, inc_value: u64) {
        // TODO: Actual error handling
        self.db_client.merge(key, inc_value.to_be_bytes()).unwrap();
    }

    pub fn get_key(&self, key: &str) -> u64 {
        // TODO: Actual error handling
        debug!("Getting key: {}", key);
        let val = self.db_client.get(key).unwrap().unwrap();
        u64::from_be_bytes(val.try_into().unwrap())
    }

    pub fn prefix_get_key(&self, key: &str) -> Vec<(Box<[u8]>, Box<[u8]>)> {
        let mut iter = self.db_client.iterator(IteratorMode::Start);
        iter.set_mode(IteratorMode::From(key.as_bytes(), Direction::Forward));

        let mut results = Vec::new();
        for item in iter {
            let (key, value) = item;
            results.push((key, value));
        }

        results
    }
}

fn increment_merge(
    _new_key: &[u8],
    existing_val: Option<&[u8]>,
    operands: &MergeOperands,
) -> Option<Vec<u8>> {
    let mut val = if let Some(old_val) = existing_val {
        // TODO: Actual error handling
        u64::from_be_bytes(old_val.try_into().unwrap())
    } else {
        0
    };

    for op in operands {
        let inc_amount = u64::from_be_bytes(op.try_into().unwrap());

        val += inc_amount;
    }

    let result = val.to_be_bytes().into_iter().collect();
    Some(result)
}

#[cfg(test)]
mod test {
    use crate::test_utils::TestDbWrapper;

    #[test]
    fn test_prefix_key_fetch() {
        let db = TestDbWrapper::new();
        let counter_db = db.get_db();

        counter_db.db_client.put("test#pre", "test").unwrap();
        counter_db.db_client.put("test#pre#2", "test").unwrap();
        counter_db.db_client.put("test#post#2", "test").unwrap();

        println!("{:?}", counter_db.db_client.get("test#post").unwrap());

        let result = counter_db.prefix_get_key("test#pre");

        assert_eq!(result.len(), 2);
        assert_eq!(std::str::from_utf8(&result[0].0).unwrap(), "test#pre");
        assert_eq!(std::str::from_utf8(&result[1].0).unwrap(), "test#pre#2");
    }
}
