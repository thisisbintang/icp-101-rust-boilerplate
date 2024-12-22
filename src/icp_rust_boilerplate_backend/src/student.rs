use candid::{Decode, Encode};
use ic_stable_structures::{BoundedStorable, Storable};
use std::borrow::Cow;

use crate::{time, Error, ID_COUNTER, STUDENT_STORAGE};

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
pub struct Student {
    id: u64,
    name: String,
    email: String,
    created_at: u64,
    updated_at: Option<u64>,
}

impl Storable for Student {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Student {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
pub struct StudentPayload {
    name: String,
    email: String,
}

#[ic_cdk::query]
fn get_all_students() -> Result<Vec<Student>, Error> {
    let students = _get_all_students();

    Ok(students)
}

fn _get_all_students() -> Vec<Student> {
    STUDENT_STORAGE.with(|students| {
        students
            .borrow()
            .iter()
            .map(|(_, value)| value.clone())
            .collect()
    })
}

#[ic_cdk::query]
fn get_student(id: u64) -> Result<Student, Error> {
    match _get_student(&id) {
        Some(message) => Ok(message),
        None => Err(Error::NotFound {
            msg: format!("a message with id={} not found", id),
        }),
    }
}

fn _get_student(id: &u64) -> Option<Student> {
    STUDENT_STORAGE.with(|s| s.borrow().get(id))
}

#[ic_cdk::update]
fn add_student(student: StudentPayload) -> Option<Student> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let student = Student {
        id,
        created_at: time(),
        updated_at: None,
        email: student.email,
        name: student.name,
    };
    do_insert(&student);
    Some(student)
}

// helper method to perform insert.
fn do_insert(message: &Student) {
    STUDENT_STORAGE.with(|service| service.borrow_mut().insert(message.id, message.clone()));
}

#[ic_cdk::update]
fn update_student(id: u64, payload: StudentPayload) -> Result<Student, Error> {
    match STUDENT_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut student) => {
            student.email = payload.email;
            student.name = payload.name;
            student.updated_at = Some(time());
            do_insert(&student);
            Ok(student)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update a message with id={}. message not found",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn delete_student(id: u64) -> Result<Student, Error> {
    match STUDENT_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(message) => Ok(message),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a message with id={}. message not found.",
                id
            ),
        }),
    }
}
