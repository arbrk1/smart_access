
use hashbrown::HashMap;
use smart_access::{At, Cps};

struct Ensure<K,V> { key: K, value: V }

impl<V> At<Ensure<usize, V>> for HashMap<usize, V> {
    type View = V;

    fn access_at<R, F>(&mut self, kv: Ensure<usize, V>, f: F) -> Option<R> where
        F: FnOnce(&mut V) -> R
    {
        if let Some(v) = self.get_mut(&kv.key) { 
            return Some(f(v)); 
        } 

        self.insert(kv.key, kv.value);
        Some(f(self.get_mut(&kv.key).unwrap()))
    }
}

fn or_insert<'a, V>(hm: &'a mut HashMap<usize,V>, k: usize, v: V) 
    -> impl Cps<View=V> + 'a 
{
    hm.at(Ensure{ key: k, value: v })
}


#[test]
fn test() {
    let mut hm = HashMap::<usize, String>::new();

    or_insert(&mut hm, 0, String::from("Hello")).touch();
    or_insert(&mut hm, 1, String::from("world")).touch();

    let mut keys = hm.keys().map(|k| k.clone()).collect::<Vec<_>>();
    
    keys.sort();

    let mut answer = String::new();

    for k in keys {
        hm.at(&k).access(|v| { answer.extend(format!("{}", v).chars()); });
    }

    assert_eq!(answer, "Helloworld");
}

