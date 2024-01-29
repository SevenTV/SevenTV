/// A simple deduplication cache for DISPATCH events.
pub struct Dedupe {
	/// An LRU cache of the last 255 events.
	/// This cache is special, because its a stack allocated LRU cache.
	/// This means that it doesn't allocate memory on the heap, and instead
	/// uses a fixed size array. This has costs that are worth considering:
	/// - The size of the array is fixed at compile time, so it can't be
	///   configured at runtime.
	/// - The operations on the cache are O(log n) for lookups and O(n) for
	///   inserts and removals, rather than O(1) for all operations.
	///
	/// The benefits of this cache are:
	/// - It doesn't allocate memory on the heap, so it keeps the byte cost per
	///   connection low.
	/// - Less memory than a heap allocated cache. This cache is 1788 bytes,
	///   while a heap allocated cache is ~8kb - ~15k per connection, which is a
	///   lot of memory.
	lru: const_lru::ConstLru<u32, (), 255, u8>,
}

impl Dedupe {
	pub fn new() -> Self {
		Self {
			lru: const_lru::ConstLru::new(),
		}
	}

	pub fn insert(&mut self, value: u32) -> bool {
		self.lru.insert(value, ()).is_none()
	}

	pub fn remove(&mut self, value: &u32) {
		self.lru.remove(value);
	}
}
