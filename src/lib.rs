//! Practice implementation of a hash table.

pub mod hash;
pub mod iter;

use self::hash::*;
use self::iter::*;

use std::borrow::Borrow;
use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::hash::{BuildHasher, Hash};


const DEFAULT_MAX_LOAD: f64 = 0.7;
const DEFAULT_GROWTH_POLICY: f64 = 2.0;
const DEFAULT_PROBING: fn(usize, usize) -> usize = |hash, i| hash + i + i*i;

const DEFAULT_INITIAL_CAPACITY: usize = 1; // not handling zero sized


/// Alias for handling buckets.
pub type Bucket<K, V> = Option<RefCell<(K, V)>>;

/// Alias for handling results of a lookup with the `find` method.
type Find<'a, K, V> = (Option<&'a RefCell<(K, V)>>, Option<usize>);


/// Parameters needed in the configuration
/// of an [`Index`] hash table.
/// 
/// # Example
/// 
/// ```
/// use std::collections::hash_map::RandomState;
/// use index::{Index, Parameters};
/// 
/// let params = Parameters {
///     max_load: 0.7,
///     growth_policy: 2.0,
///     hasher_builder: RandomState::new(),
///     probe: |hash, i| (hash as f64 + (i as f64 / 2.0) + ((i*i) as f64 / 2.0)) as usize,
/// };
/// 
/// let mut index = Index::with_capacity_and_parameters(10, params);
/// 
/// index.insert("key", "value");
/// ```
/// 
/// [`Index`]: struct.Index.html
#[derive(Debug, Clone)]
pub struct Parameters<S> {
    /// Maximum load factor accepted before the table is resized. Default is `0.7`.
    pub max_load: f64,

    /// Ratio by which the table's capacity is grown. Default is `2`.
    pub growth_policy: f64,

    /// Hasher builder (see [`BuildHasher`]). Default is [`IndexHasherBuilder`]
    /// 
    /// [`IndexHasherBuilder`]: hash/struct.IndexHasherBuilder.html
    /// [`BuildHasher`]: https://doc.rust-lang.org/std/hash/trait.BuildHasher.html
    pub hasher_builder: S,

    /// Open addressing probing policy. Default is quadratic probing: `hash + i + i*i`
    pub probe: fn(hash: usize, i: usize) -> usize,
}


/// Simple implementation of a hash table using safe-rust.
/// 
/// The collisions are resolved through open adressing with
/// quadratic probing (although it is possible to use linear probing or other types
/// when specifying parameters).
/// 
/// # Example
/// 
/// ```
/// use index::Index;
/// 
/// let mut index = Index::new();
/// 
/// assert_eq!(index.len(), 0);
/// assert_eq!(index.capacity(), 1);
/// 
/// index.insert("salutation", "Hello, world!");
/// index.insert("ferris", "https://www.rustacean.net/more-crabby-things/dancing-ferris.gif");
/// index.insert("did you know ?", "Rust is kinda cool guys !");
/// index.insert("key", "value");
/// 
/// println!("{}", index.get("salutation").unwrap());
/// 
/// assert_eq!(index.len(), 4);
/// assert_eq!(index.capacity(), 8);
/// ```
#[derive(Clone)]
pub struct Index<K, V, S = IndexHasherBuilder> {
    params: Parameters<S>,
    capacity: usize,
    len: usize,
    table: Vec<Bucket<K, V>>,
}

impl<K, V> Index<K, V, IndexHasherBuilder>
where
    K: Hash + Eq,
{
    /// Creates an empty `Index` with default initial capacity and default parameters.
    ///
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index: Index<String, Vec<i32>> = Index::new();
    /// ```
    pub fn new() -> Index<K, V, IndexHasherBuilder> {
        Self::with_capacity(DEFAULT_INITIAL_CAPACITY)
    }

    /// Creates an empty `Index` with specified capacity and default parameters.
    ///
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index: Index<String, Vec<i32>> = Index::with_capacity(1312);
    /// ```
    pub fn with_capacity(capacity: usize) -> Index<K, V, IndexHasherBuilder> {
        Index::with_capacity_and_parameters(
            capacity,
            Parameters {
                max_load: DEFAULT_MAX_LOAD,
                growth_policy: DEFAULT_GROWTH_POLICY,
                hasher_builder: IndexHasherBuilder {},
                probe: DEFAULT_PROBING,
            },
        )
    }
}

impl<K, V, S> Index<K, V, S> {

    /// Returns the maximum load factor accepted before the table is resized.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index: Index<String, Vec<i32>> = Index::new();
    /// 
    /// assert_eq!(index.max_load(), 0.7); // default max load
    /// ```
    pub fn max_load(&self) -> f64 {
        self.params.max_load
    }

    /// Returns the ratio by which the table's capacity is grown.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index: Index<String, Vec<i32>> = Index::new();
    /// 
    /// assert_eq!(index.growth_policy(), 2.0); // default growth policy
    /// ```
    pub fn growth_policy(&self) -> f64 {
        self.params.growth_policy
    }

    /// Returns a reference to the hasher builder used in the `Index`.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// use index::hash::IndexHasherBuilder;
    /// use std::any::{Any, TypeId};
    /// 
    /// let mut index: Index<String, Vec<i32>> = Index::new();
    /// 
    /// assert_eq!(index.hasher().type_id(), TypeId::of::<IndexHasherBuilder>()) // default hasher builder
    /// ```
    pub fn hasher(&self) -> &S {
        &self.params.hasher_builder
    }

    /// Returns the probing function pointer of the `Index`.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index: Index<String, Vec<i32>> = Index::new();
    /// 
    /// let p = |h: usize, i: usize| h + i + i*i; // default prober
    /// 
    /// assert_eq!((index.probe())(45, 2), p(45, 2));
    /// ```
    pub fn probe(&self) -> fn(usize, usize) -> usize {
        self.params.probe
    }


    /// Returns the capacity of the `Index`.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index: Index<&str, &str> = Index::with_capacity(6);
    /// 
    /// assert_eq!(index.len(), 0);
    /// assert_eq!(index.capacity(), 6);
    /// ```
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the number of elements in the `Index`.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index: Index<&str, i32> = Index::with_capacity(6);
    /// 
    /// index.insert("one", 1);
    /// index.insert("two", 2);
    /// index.insert("three", 3);
    /// 
    /// assert_eq!(index.len(), 3);
    /// assert_eq!(index.capacity(), 6);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the `Index` contains no elements.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index: Index<&str, &str> = Index::with_capacity(10);
    /// 
    /// assert!(index.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the current load factor of the `Index`.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index: Index<&str, i32> = Index::with_capacity(6);
    /// 
    /// index.insert("one", 1);
    /// index.insert("two", 2);
    /// index.insert("three", 3);
    /// 
    /// assert_eq!(index.load(), 0.5);
    /// ```
    pub fn load(&self) -> f64 {
        (self.len as f64) / (self.capacity as f64)
    }

    /// Clear the `Index`, replacing all entries with empty buckets.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// use index::Bucket;
    /// 
    /// let mut index: Index<&str, i32> = Index::with_capacity(6);
    /// 
    /// index.insert("one", 1);
    /// index.insert("two", 2);
    /// index.insert("three", 3);
    /// 
    /// index.clear();
    /// 
    /// assert!(index.get("two").is_none());
    /// assert_eq!(index.len(), 0);
    /// 
    /// ```
    pub fn clear(&mut self) {
        for entry in self.table.iter_mut() {
            *entry = Bucket::None;
        }
        self.len = 0;
    }

    /// Returns an iterator over the keys of the `Index`. 
    /// The iterator's associated type is `Ref<'a, K>`.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index = Index::with_capacity(10);
    /// 
    /// index.insert("salutation", "Hello, world!");
    /// index.insert("ferris", "https://www.rustacean.net/more-crabby-things/dancing-ferris.gif");
    /// index.insert("did you know ?", "Rust is kinda cool !");
    /// 
    /// for key in index.keys() {
    ///     println!("key: {:?}", key);
    /// }
    /// 
    /// assert_eq!(index.len(), index.keys().count());
    /// ```
    pub fn keys(&self) -> Keys<K, V> {
        Keys::new(&self.table)
    }

    /// Returns an iterator over the values of the `Index`. 
    /// The iterator's associated type is `Ref<'a, V>`.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index = Index::with_capacity(10);
    /// 
    /// index.insert("salutation", "Hello, world!");
    /// index.insert("ferris", "https://www.rustacean.net/more-crabby-things/dancing-ferris.gif");
    /// index.insert("did you know ?", "Rust is kinda cool !");
    /// 
    /// for value in index.values() {
    ///     println!("value: {:?}", value);
    /// }
    /// 
    /// assert_eq!(index.len(), index.values().count());
    /// ```
    pub fn values(&self) -> Values<K, V> {
        Values::new(&self.table)
    }

    /// Returns a mutable iterator over the values of the `Index`. 
    /// The iterator's associated type is `RefMut<'a, V>`.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index = Index::with_capacity(10);
    /// 
    /// index.insert("salutation", "Hello, world!");
    /// index.insert("ferris", "https://www.rustacean.net/more-crabby-things/dancing-ferris.gif");
    /// index.insert("did you know ?", "Rust is kinda cool !");
    /// 
    /// for mut value in index.values_mut() {
    ///     *value = "overwritten!";
    /// }
    /// 
    /// assert_eq!(*index.get("ferris").unwrap(), "overwritten!");
    /// 
    /// ```
    pub fn values_mut(&self) -> ValuesMut<K, V> {
        ValuesMut::new(&self.table)
    }

    /// Return an iterator over the key-value pairs of the `Index`.
    /// The iterator's associated type is `Ref<'a, (K, V)>`.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index = Index::with_capacity(10);
    /// 
    /// index.insert("salutation", "Hello, world!");
    /// index.insert("ferris", "https://www.rustacean.net/more-crabby-things/dancing-ferris.gif");
    /// index.insert("did you know ?", "Rust is kinda cool !");
    /// 
    /// for entry in index.iter() {
    ///     println!("key: {:?} => value: {:?}", entry.0, entry.1);
    /// }
    /// 
    /// assert_eq!(index.len(), index.iter().count());
    /// ```
    pub fn iter(&self) -> Iter<K, V> {
        Iter::new(&self.table)
    }

    /// Return a mutable iterator over the key-value pairs of the `Index`.
    /// The iterator's associated type is `RefMut<'a, (K, V)>`.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index = Index::with_capacity(10);
    /// 
    /// index.insert("salutation", "Hello, world!");
    /// index.insert("ferris", "https://www.rustacean.net/more-crabby-things/dancing-ferris.gif");
    /// index.insert("did you know ?", "Rust is kinda cool !");
    /// 
    /// for mut entry in index.iter_mut() {
    ///     entry.1 = entry.0.clone();
    /// }
    /// 
    /// assert_eq!(*index.get("ferris").unwrap(), "ferris");
    /// ```
    pub fn iter_mut(&self) -> IterMut<K, V> {
        IterMut::new(&self.table)
    }

    /// Returns iterator taking ownership and moving out the key-value pairs of the `Index`.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index = Index::with_capacity(10);
    /// 
    /// index.insert("salutation", "Hello, world!");
    /// index.insert("ferris", "https://www.rustacean.net/more-crabby-things/dancing-ferris.gif");
    /// index.insert("did you know ?", "Rust is kinda cool !");
    /// 
    /// let v: Vec<(&str, &str)> = index.drain().collect();
    /// 
    /// assert_eq!(index.len(), 0);
    /// assert_eq!(v.len(), 3);
    /// assert!(v.contains(&("salutation", "Hello, world!")));
    /// ```
    pub fn drain(&mut self) -> Drain<K, V> {
        Drain::new(&mut self.table, &mut self.len)
    }
}

impl<K, V, S> Index<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher + Clone,
{
    // static

    /// Creates an empty `Index` with specified capacity and parameters.
    /// 
    /// See [`Parameters`] for details.
    /// 
    /// # Example
    /// 
    /// ```
    /// use std::collections::hash_map::RandomState;
    /// use index::{Index, Parameters};
    /// 
    /// let params = Parameters {
    ///     max_load: 0.7,
    ///     growth_policy: 2.0,
    ///     hasher_builder: RandomState::new(),
    ///     probe: |hash, i| (hash as f64 + (i as f64 / 2.0) + ((i*i) as f64 / 2.0)) as usize,
    /// };
    /// 
    /// let mut index = Index::with_capacity_and_parameters(10, params);
    /// 
    /// index.insert("key", "value");
    /// ```
    /// 
    /// [`Parameters`]: struct.Parameters.html
    pub fn with_capacity_and_parameters(capacity: usize, params: Parameters<S>) -> Index<K, V, S> {
        
        let capacity = if capacity == 0 { DEFAULT_INITIAL_CAPACITY } else { capacity };
        
        let mut index = Index {
            params,
            capacity,
            len: 0,
            table: Vec::with_capacity(capacity),
        };

        Self::init_table(&mut index.table, index.capacity);

        index
    }

    /// Initializes inner table with empty buckets according to specified capacity.
    fn init_table(table: &mut Vec<Bucket<K, V>>, capacity: usize) {
        for _ in 0..capacity {
            table.push(Bucket::None);
        }

        // useless but that paranoia
        assert_eq!(capacity, table.len());
        assert_eq!(capacity, table.capacity());
    }

    // methods

    /// Resizes `Index` with new capacity by allocating a new `Index`
    /// and moving entries from the old one to the new one by using insert to
    /// rehash the entries (if the new capacity is to small, the insert operation will grow
    /// the new `Index` automatically).
    fn resize(&mut self, new_capacity: usize) {
        let mut new_index = Self::with_capacity_and_parameters(
            new_capacity,
            self.params.clone(),
        );

        for (key, value) in self.drain() {
            new_index.insert(key, value);
        }

        *self = new_index;
    }

    /// Grows `Index` according to growth policy.
    fn grow(&mut self) {
        let new_cap = (self.capacity as f64 * self.params.growth_policy) as usize;
        self.resize(new_cap);
    }

    /// Searches for an entry according to specified hash and discriminating closure.
    /// 
    /// See alias definition of `Find<'a, K, V>` at the top of this file for more details.
    fn find<F>(&self, hash: usize, f: F) -> Find<K, V>
    where
        F: Fn(Ref<(K, V)>) -> bool,
    {
        for i in 0..self.capacity {
            let probe = (self.params.probe)(hash, i) % self.capacity;

            match &self.table[probe] {
                Some(pair) if f(pair.borrow()) => return (Some(pair), Some(probe)), // found matching bucket
                None => return (None, Some(probe)), // found empty bucket
                Some(_) => continue,
            }
        }

        (None, None) // found nothing
    }


    /// Inserts key-value pair in the `Index`.
    /// 
    /// If it encounters an occupied bucket with the same key, it will replace the
    /// entry according to the new value and return the old bucket.
    /// 
    /// The function also verifies before anything else that the load factor is lesser
    /// than the maximum accepted load, if not it will grow the `Index` before proceeding to the insertion.
    /// 
    /// If the lookup returns no valid result, the insertion is considered impossible and 
    /// the function will grow the `Index` and retry to insert the pair.
    /// 
    /// # Example
    /// 
    /// ```
    /// use index::Index;
    /// 
    /// let mut index = Index::with_capacity(2);
    /// 
    /// index.insert("key", "value");
    /// 
    /// assert_eq!(*index.get("key").unwrap(), "value");
    /// 
    /// index.insert("key", "new value");
    /// 
    /// assert_eq!(*index.get("key").unwrap(), "new value");
    /// 
    /// assert_eq!(index.len(), 1);
    /// assert_eq!(index.capacity(), 2);
    /// 
    /// index.insert("salutation", "Hello, world!");
    /// index.insert("ferris", "https://www.rustacean.net/more-crabby-things/dancing-ferris.gif");
    /// index.insert("did you know ?", "Rust is kinda cool guys !");
    /// 
    /// assert_eq!(index.len(), 4);
    /// assert_eq!(index.capacity(), 8);
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> Bucket<K, V> {
        let hash = make_hash(&self.params.hasher_builder, &key) as usize;

        if self.load() >= self.params.max_load {
            self.grow();
        }

        match self.find(hash, |p| key.eq(&p.0)) {
            (Some(_), Some(i)) => {
                std::mem::replace(&mut self.table[i], Bucket::Some(RefCell::new((key, value))))
            }
            (None, Some(i)) => {
                self.table[i] = Bucket::Some(RefCell::new((key, value)));
                self.len += 1;
                Bucket::None
            }
            _ => {
                self.grow();
                self.insert(key, value)
            }
        }
    }

    // pub fn remove_entry<Q>(&mut self, key: &Q) -> Bucket<K, V> where K: Borrow<Q>, Q: Hash + Eq + ?Sized
    /*
        Problem: removing entry can corrupt lookup integrity
                 (find may encounter empty bucket before searched value)

        Solutions:
            - use find_match and find_empty
                Problem: find_match will always have to be used for remove and get operations
                         to ensure lookup integrity and will have O(n) complexity if key isnt in table (because wont return first empty bucket found)
            - use flag array for present, empty, removed values ?

        Same problem arises when modifying keys through an IterMut
    */

    /// Returns a reference to the value associated with the specified key
    /// if the lookup found a match, else it returns `None`.
    /// 
    /// # Example
    ///  
    /// ```
    /// use index::Index;
    /// 
    /// let mut index = Index::with_capacity(10);
    /// 
    /// index.insert("salutation", "Hello, world!");
    /// index.insert("ferris", "https://www.rustacean.net/more-crabby-things/dancing-ferris.gif");
    /// index.insert("did you know ?", "Rust is kinda cool !");
    /// 
    /// assert_eq!(*index.get("salutation").unwrap(), "Hello, world!");
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<Ref<V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let hash = make_hash(self.hasher(), &key) as usize;
        self.find(hash, |p| key.borrow().eq(p.0.borrow()))
            .0
            .map(|pair| Ref::map(pair.borrow(), |p| &p.1))
    }

    /// Returns a mutable reference to the value associated with the specified key
    /// if the lookup found a match, else it returns `None`.
    /// 
    /// # Example
    ///  
    /// ```
    /// use index::Index;
    /// 
    /// let mut index = Index::with_capacity(10);
    /// 
    /// index.insert("salutation", "Hello, world!");
    /// index.insert("ferris", "https://www.rustacean.net/more-crabby-things/dancing-ferris.gif");
    /// index.insert("did you know ?", "Rust is kinda cool !");
    /// 
    /// *index.get_mut("salutation").unwrap() = "Hello, rust!";
    /// 
    /// assert_eq!(*index.get("salutation").unwrap(), "Hello, rust!");
    /// ```
    pub fn get_mut<Q>(&self, key: &Q) -> Option<RefMut<V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let hash = make_hash(self.hasher(), &key) as usize;
        self.find(hash, |p| key.eq(p.0.borrow()))
            .0
            .map(|pair| RefMut::map(pair.borrow_mut(), |p| &mut p.1))
    }

    /// Returns a reference to the key-value pair associated with the specified key
    /// if the lookup found a match, else it returns `None`.
    /// 
    /// # Example
    ///  
    /// ```
    /// use index::Index;
    /// 
    /// let mut index = Index::with_capacity(10);
    /// 
    /// index.insert("salutation", "Hello, world!");
    /// index.insert("ferris", "https://www.rustacean.net/more-crabby-things/dancing-ferris.gif");
    /// index.insert("did you know ?", "Rust is kinda cool !");
    /// 
    /// assert_eq!(*index.get_pair("did you know ?").unwrap(), ("did you know ?", "Rust is kinda cool !"));
    /// ```
    pub fn get_pair<Q>(&self, key: &Q) -> Option<Ref<(K, V)>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let hash = make_hash(self.hasher(), &key) as usize;
        self.find(hash, |p| key.eq(p.0.borrow()))
            .0
            .map(|pair| pair.borrow())
    }
}

impl<K, V, S> fmt::Debug for Index<K, V, S>
where
    K: fmt::Debug,
    V: fmt::Debug,
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = format!(
            "Index {{\n\tparams: {:?}\t\ncapacity: {:?}\n\tlen: {:?}\n\ttable:\n\t[",
            self.params, self.capacity, self.len
        );

        for (i, entry) in self.table.iter().enumerate() {
            s = format!(
                "{}\n\t\t{} : {:?},",
                s,
                i,
                if let Some(pair) = entry {
                    Some(pair.borrow())
                } else {
                    None
                }
            );
        }
        s = format!("{}\n\t]\n}}", s);

        write!(f, "{}", s)
    }
}

impl<K, V> Default for Index<K, V, IndexHasherBuilder>
where
    K: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}
