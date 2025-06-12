use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead};

const ENROLMENTS_PATH: &str = "enrolments.psv"; // File path for the enrolments.psv dataset


fn main() -> io::Result<()> {
    let file = File::open(ENROLMENTS_PATH)?; // Open the file
    let reader = io::BufReader::new(file); // Use a buffered reader for better performance

    // HashSet to store unique students by their student number
    let mut unique_students: HashSet<String> = HashSet::new();
    
    // HashMap to store the count of students per course
    let mut course_counts: HashMap<String, u32> = HashMap::new();
    
    // Variables to calculate the total WAM and the number of students
    let mut total_wam: f32 = 0.0;
    let mut total_students: u32 = 0;

    // HashSet to track students whose WAM has already been processed
    let mut processed_students: HashSet<String> = HashSet::new();

    // Process each line in the file
    for line in reader.lines() {
        let line = line?;  // Read each line from the file
        let fields: Vec<&str> = line.split('|').collect(); // Split each line into fields using '|' as delimiter

        // Skip rows with malformed data (less than 6 fields)
        if fields.len() < 6 {
            continue;
        }

        // Extract relevant information from each field
        let student_number = fields[1].to_string(); // Student number (field 1)
        let course_code = fields[0].to_string();   // Course code (field 0)
        let wam: f32 = fields[5].parse().unwrap_or(0.0); // Parse WAM (default to 0 if invalid)

        // Process the student only if their WAM has not been processed before
        if !processed_students.contains(&student_number) {
            // Add WAM to total WAM and increment student count
            total_wam += wam;
            total_students += 1;

            // Mark this student as processed
            processed_students.insert(student_number.clone());
        }

        // Count the number of students per course
        let counter = course_counts.entry(course_code.clone()).or_insert(0);
        *counter += 1;

        // Add the student to the set of unique students
        unique_students.insert(student_number);
    }

    // Calculate the number of unique students
    let num_unique_students = unique_students.len();

    // Find the most common and least common courses based on student counts
    let most_common_course = course_counts.iter().max_by_key(|entry| entry.1).unwrap();
    let least_common_course = course_counts.iter().min_by_key(|entry| entry.1).unwrap();

    // Calculate the average WAM
    let average_wam = if total_students > 0 {
        total_wam / total_students as f32
    } else {
        0.0 // If no students are processed, return 0.0
    };

    // Output the results
    println!("Number of students: {}", num_unique_students);
    println!(
        "Most common course: {} with {} students",
        most_common_course.0, most_common_course.1
    );
    println!(
        "Least common course: {} with {} students",
        least_common_course.0, least_common_course.1
    );
    println!("Average WAM: {:.2}", average_wam);

    Ok(())
}
