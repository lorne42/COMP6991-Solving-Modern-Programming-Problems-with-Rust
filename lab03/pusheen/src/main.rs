fn main() {
    let mut vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    {
        let a = &mut vec;  // First mutable reference
        a.push(11);        // Modify the vector
    } // `a` goes out of scope here

    {
        let b = &mut vec;  // Now it's safe to create a new mutable reference
        b.push(12);
    } // `b` goes out of scope
}
/*This program fails to compile due to Rust's borrowing rules, 
which prevent multiple mutable references to the same variable 
at the same time.

The line let a = &mut vec; creates a mutable reference to vec.
The line let b = &mut vec; attempts to create another mutable reference to vec while a is still in scope.
Rust does not allow multiple mutable references to the same value at the same time because this 
could lead to data races or undefined behavior.
*/

/*
How I Fixed the Program

Introduced scopes {} to ensure that only one mutable reference exists at a time.
The first mutable reference a is created and used inside a block {}.
After a goes out of scope, the second mutable reference b is created in a separate block.*/