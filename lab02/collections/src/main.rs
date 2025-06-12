use std::collections::VecDeque;

const MAX_ITER: i32 = 300000;

fn main() {
    // Vectors
    vec_operations();

    // VecDeque
    vec_deque_operations();

    // TODO: your code here, for linked list insertions
    // LinkedList
    linked_list_operations();

    // HashMap
    hash_map_operations();
    // TODO: your code here, for hashmap insertions

    // TODO: your text explanation to the questions in the spec
    /*
    1 Likely, VecDeque will perform well for both operations 
    because it allows fast insertion and removal at both ends. 
    Vec and LinkedList may perform similarly for insertion 
    but could be slower for removal (especially Vec for front removal).
    
    1.1 VecDeque is optimized for both front and back operations due to its circular buffer. 
    In contrast, Vec and LinkedList require shifting or traversing nodes for operations 
    that aren't at the end of the collection. HashMap has a constant-time lookup and removal 
    due to the hash table implementation.
    
    2 Yes, there is a significant difference. In Vec, removing an element from the front involves 
    shifting all other elements down, which results in O(n) time. In VecDeque, removal from the front is O(1) 
    because of the circular buffer design.

    3 Should use VecDeque when you need to perform frequent insertions and removals at both the front and 
    back of the collection. Vec is better when you only need to add elements at the end.

    4 LinkedList can be useful if you need efficient insertions and removals at both ends, 
    and when you don't care about random access speed 
    (because accessing elements is O(n)). 
    It may be useful for cases where elements are being frequently added 
    or removed without needing to iterate over them all.

    5 The results were likely expected. VecDeque would perform well for both adding 
    and removing from both ends due to its circular buffer structure. 
    LinkedList was also efficient for removal from the front, 
    but it is slower than VecDeque for access operations. HashMap performed well for 
    insertion and removal due to its O(1) average-time complexity.
    */
}

/// measure the insertion and removal
/// operations of a vector
fn vec_operations() {
    let mut vec = Vec::new();

    let time_start = std::time::Instant::now();
    for i in 0..MAX_ITER {
        vec.push(i);
    }
    let time_end = std::time::Instant::now();

    println!("==== Vector ====");
    println!("insert: {:?}", time_end - time_start);

    let time_start = std::time::Instant::now();
    for _ in 0..MAX_ITER {
        vec.remove(0);
    }
    let time_end = std::time::Instant::now();

    println!("remove: {:?}", time_end - time_start);
}

/// measure the insertion and removal
/// operations of a VecDeque
fn vec_deque_operations() {
    let mut vec_deque = VecDeque::new();

    let time_start = std::time::Instant::now();
    for i in 0..MAX_ITER {
        vec_deque.push_back(i);
    }
    let time_end = std::time::Instant::now();

    println!("==== VecDeque ====");
    println!("insert: {:?}", time_end - time_start);

    let time_start = std::time::Instant::now();
    for _ in 0..MAX_ITER {
        vec_deque.pop_front();
    }
    let time_end = std::time::Instant::now();

    println!("remove: {:?}", time_end - time_start);
}

fn linked_list_operations() {
    let mut linked_list = LinkedList::new();

    let time_start = Instant::now();
    for i in 0..MAX_ITER {
        linked_list.push_back(i);  // O(1) insertion at the back
    }
    let time_end = Instant::now();
    println!("==== LinkedList ====");
    println!("insert: {:?}", time_end - time_start);

    let time_start = Instant::now();
    for _ in 0..MAX_ITER {
        linked_list.pop_front();  // O(1) removal from the front
    }
    let time_end = Instant::now();
    println!("remove: {:?}", time_end - time_start);
}

fn hash_map_operations() {
    let mut hash_map = HashMap::new();

    let time_start = Instant::now();
    for i in 0..MAX_ITER {
        hash_map.insert(i, i);  // O(1) insertion
    }
    let time_end = Instant::now();
    println!("==== HashMap ====");
    println!("insert: {:?}", time_end - time_start);

    let time_start = Instant::now();
    for i in 0..MAX_ITER {
        hash_map.remove(&i);  // O(1) removal
    }
    let time_end = Instant::now();
    println!("remove: {:?}", time_end - time_start);
}