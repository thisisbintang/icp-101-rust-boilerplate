use candid::{Decode, Encode};
use ic_stable_structures::{BoundedStorable, Storable};
use std::borrow::Cow;

use crate::{time, Error, ID_COUNTER, STUDENT_STORAGE};

// Define the Student struct to represent a student in the system.
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
pub struct Student {
    id: u64,
    name: String,
    email: String,
    created_at: u64,
    updated_at: Option<u64>,
}

// Implement serialization and deserialization for Student.
impl Storable for Student {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Set limits for Student storage size and flexibility.
impl BoundedStorable for Student {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Define the payload structure for creating or updating a student.
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
pub struct StudentPayload {
    name: String,
    email: String,
}

// Retrieve all students from the storage.
#[ic_cdk::query]
fn get_all_students() -> Result<Vec<Student>, Error> {
    let students = _get_all_students();
    Ok(students)
}

// Internal function to fetch all students as a vector.
fn _get_all_students() -> Vec<Student> {
    STUDENT_STORAGE.with(|students| {
        students
            .borrow()
            .iter()
            .map(|(_, value)| value.clone())
            .collect()
    })
}

// Retrieve a specific student by their ID.
#[ic_cdk::query]
fn get_student(id: u64) -> Result<Student, Error> {
    match _get_student(&id) {
        Some(student) => Ok(student),
        None => Err(Error::NotFound {
            msg: format!("A student with id={} not found.", id),
        }),
    }
}

// Internal function to fetch a student by ID.
fn _get_student(id: &u64) -> Option<Student> {
    STUDENT_STORAGE.with(|s| s.borrow().get(id))
}

// Add a new student to the registry.
#[ic_cdk::update]
fn add_student(payload: StudentPayload) -> Result<Student, Error> {
    // Validate the input payload.
    if payload.name.trim().is_empty() || payload.email.trim().is_empty() {
        return Err(Error::InvalidInput {
            msg: "Name and email cannot be empty.".to_string(),
        });
    }

    // Generate a new unique ID for the student.
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    // Create the new student with the provided payload.
    let student = Student {
        id,
        name: payload.name,
        email: payload.email,
        created_at: time(),
        updated_at: None,
    };

    // Insert the student into storage.
    do_insert(&student);
    Ok(student)
}

// Helper function to insert a student into storage.
fn do_insert(student: &Student) {
    STUDENT_STORAGE.with(|service| service.borrow_mut().insert(student.id, student.clone()));
}

// Update an existing student's details by ID.
#[ic_cdk::update]
fn update_student(id: u64, payload: StudentPayload) -> Result<Student, Error> {
    // Validate the input payload.
    if payload.name.trim().is_empty() || payload.email.trim().is_empty() {
        return Err(Error::InvalidInput {
            msg: "Name and email cannot be empty.".to_string(),
        });
    }

    // Fetch the student from storage and update their details.
    match STUDENT_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut student) => {
            student.name = payload.name;
            student.email = payload.email;
            student.updated_at = Some(time());
            do_insert(&student); // Save the updated student back to storage.
            Ok(student)
        }
        None => Err(Error::NotFound {
            msg: format!("Couldn't update a student with id={}. Student not found.", id),
        }),
    }
}

// Delete a student by ID from the registry.
#[ic_cdk::update]
fn delete_student(id: u64) -> Result<Student, Error> {
    // Remove the student from storage.
    match STUDENT_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(student) => Ok(student),
        None => Err(Error::NotFound {
            msg: format!("Couldn't delete a student with id={}. Student not found.", id),
        }),
    }
}
