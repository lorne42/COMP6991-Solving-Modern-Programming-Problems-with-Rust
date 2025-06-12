1) I saw someone's code fail to compile because they 
were trying to send non-thread-safe data across threads. 
How does the Rust language allow for static (i.e. at compile time)
guarantees that specific data can be sent/shared acrosss threads?

Answer:Rust uses marker traits like Send and Sync, along with its type system, to enforce these guarantees statically. When you try to move data into a thread (e.g., via thread::spawn), the compiler checks whether the data implements Send. If it doesn't, the code won't compile.


2) Do you have to then implement the Send and Sync traits for 
every piece of data (i.e. a struct) you want to share and send across threads?

Answer:
Usually, no. Rust will automatically derive Send and Sync for your types if all their fields are Send or Sync. You only need to manually implement them in rare cases, typically when you're working with unsafe code or low-level primitives like raw pointers.

3) What types in the course have I seen that aren't Send? Give one example, 
and explain why that type isn't Send 

Answer:
A common one is Rc<T>. It’s not Send because it’s a reference-counted pointer meant for single-threaded use. It doesn’t use atomic operations, so moving it between threads could cause race conditions. You’d use Arc<T> instead for thread-safe reference counting.

4) What is the relationship between Send and Sync? Does this relate
to Rust's Ownership system somehow?

Answer:
Yes, closely.

Send means a value can be transferred to another thread.

Sync means a reference to the value (&T) can be safely shared between threads.

These traits work with Rust’s ownership and borrowing rules to prevent data races at compile time.

5) Are there any types that could be Send but NOT Sync? Is that even possible?

Answer:
Yes, it’s possible. A type can be safely moved to another thread (Send) but still not safe to access from multiple threads at the same time (!Sync). For example, mpsc::Sender<T> is Send but not Sync, since you can move the sender to another thread, but you shouldn't use the same sender from multiple threads without synchronization.


6) Could we implement Send ourselves using safe rust? why/why not?

Answer:
No, not with safe Rust. Implementing Send (or Sync) manually requires unsafe because you're promising the compiler that you know it's safe to share or send that type across threads. Safe Rust doesn't give you that level of control, which is why the compiler won’t let you write impl Send without using unsafe.
