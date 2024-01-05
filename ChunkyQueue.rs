use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use bit_set::BitSet;

type Value = i32;
const CHUNK_SIZE: usize = 64;  // Adjust based on your needs

/// Represents a chunk of storage.
///
/// A chunk contains a fixed-size array of optional values and a free list of indices pointing
/// to unoccupied slots within the array.
struct Chunk {
    /// Storage for values. `None` indicates an unoccupied slot.
    values: [Option<Value>; CHUNK_SIZE],
    /// A list of indices pointing to unoccupied slots within the `values` array.
    free_indices: VecDeque<usize>,
}

impl Chunk {
    /// Constructs a new `Chunk` with all slots unoccupied.
    fn new() -> Self {
        Chunk {
            values: Default::default(),
            free_indices: (0..CHUNK_SIZE).collect(),
        }
    }

    /// Tries to allocate the given value within the chunk.
    ///
    /// Returns the index within the chunk where the value was stored, or `None` if the chunk is full.
    fn allocate(&mut self, value: Value) -> Option<usize> {
        self.free_indices.pop_front().map(|index| {
            self.values[index] = Some(value);
            index
        })
    }

    /// Deallocates the value at the given index within the chunk.
    ///
    /// If the provided index is out of bounds, this function does nothing.
    fn deallocate(&mut self, index: usize) {
        if index < CHUNK_SIZE {
            self.values[index] = None;
            self.free_indices.push_back(index);
        }
    }

    /// Checks if the chunk has any unoccupied slots.
    ///
    /// Returns `true` if there's at least one free slot, otherwise `false`.
    fn has_free_slot(&self) -> bool {
        !self.free_indices.is_empty()
    }
}

lazy_static! {
    /// A dynamic list of chunks for storing values.
    static ref CHUNKS: Arc<Mutex<Vec<Chunk>>> = Arc::new(Mutex::new(Vec::new()));
    /// A bit set tracking chunks with free slots. 
    /// A set bit at index `i` indicates that `CHUNKS[i]` has at least one free slot.
    static ref FREE_CHUNKS: Arc<Mutex<BitSet>> = Arc::new(Mutex::new(BitSet::new()));
}

/// Allocates the given value and returns its location as a (chunk_index, value_index) tuple.
///
/// If no chunks with free slots are available, a new chunk is created.
fn allocate_value(value: Value) -> (usize, usize) {
    let mut chunks = CHUNKS.lock().unwrap();
    let mut free_chunks = FREE_CHUNKS.lock().unwrap();

    // Find a chunk with a free slot
    let chunk_index = free_chunks.iter().next()
        .unwrap_or_else(|| {
            let new_chunk = Chunk::new();
            chunks.push(new_chunk);
            let index = chunks.len() - 1;
            free_chunks.insert(index);
            index
        });

    let chunk = &mut chunks[chunk_index];
    let value_index = chunk.allocate(value).unwrap();

    if !chunk.has_free_slot() {
        free_chunks.remove(chunk_index);
    }

    (chunk_index, value_index)
}

/// Deallocates the value at the specified location.
///
/// If the provided chunk_index is out of bounds, this function does nothing.
fn deallocate_value(chunk_index: usize, value_index: usize) {
    let mut chunks = CHUNKS.lock().unwrap();
    let mut free_chunks = FREE_CHUNKS.lock().unwrap();

    if let Some(chunk) = chunks.get_mut(chunk_index) {
        chunk.deallocate(value_index);
        if chunk.has_free_slot() {
            free_chunks.insert(chunk_index);
        }
    }
}

fn main() {
    let (chunk_index, value_index) = allocate_value(42);
    println!("Value allocated in chunk {} at index {}", chunk_index, value_index);
    deallocate_value(chunk_index, value_index);
}
