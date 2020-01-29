mod fixed;
pub use self::fixed::IndexFixed;
use crate::{
    memorefhead::MemoRefHead
};

trait Index{
    fn insert(&self, key: u64, head: MemoRefHead);
    fn get(&self, key: u64) -> Option<MemoRefHead>;
}

#[cfg(test)]
mod test {
    use crate::{
        SubjectHandle,
        index::{
            IndexFixed
        },
        Network,
        Slab,
        util::simulator::Simulator
    };

    use std::collections::HashMap;

    #[unbase_test_util::async_test]
    async fn index_construction() {
        let net = Network::create_new_system();
        let simulator = Simulator::new();
        net.add_transport(Box::new(simulator.clone()));

        let slab_a = Slab::new(&net);
        let context_a = slab_a.create_context();

        // Create a new fixed tier index (fancier indexes not necessary for the proof of concept)

        let mut index = IndexFixed::new(&context_a, 5);

        assert_eq!(context_a.is_fully_materialized(), true);

        // First lets do a single index test
        let i = 1234;
        let mut vals = HashMap::new();
        vals.insert("record number".to_string(), i.to_string());

        let record = SubjectHandle::new(&context_a, vals).await.unwrap();
        index.insert(&context_a, i, record.head.clone()).await.unwrap();

        assert_eq!(index.get(&context_a,1234).await.unwrap().unwrap().get_value(&context_a.slab, "record number").await.unwrap().unwrap(), "1234");

        // Ok, now lets torture it a little
        for i in 0..10 {
            let mut vals = HashMap::new();
            vals.insert("record number".to_string(), i.to_string());

            let record = SubjectHandle::new(&context_a, vals).await.unwrap();
            index.insert(&context_a, i, record.head.clone()).await.unwrap();
        }

        for i in 0..10 {
            assert_eq!(index.get(&context_a, i).await.unwrap().unwrap().get_value(&context_a.slab, "record number").await.unwrap().unwrap(), i.to_string());
        }

        //assert_eq!( context_a.is_fully_materialized(), false );
        //context_a.fully_materialize();
    }
}