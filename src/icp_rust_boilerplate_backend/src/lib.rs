#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Student {
    id: u64,
    name: String,
    age: String,
    hobby: String,
    email: String,
    created_at: u64,
    updated_at: Option<u64>,
}

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for Student {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for Student {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static STORAGE: RefCell<StableBTreeMap<u64, Student, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct StudentPayload {
    name: String,
    age: String,
    hobby: String,
    email: String,
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
    STORAGE.with(|s| s.borrow().get(id))
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
        age: student.age,
        email: student.email,
        hobby: student.hobby,
        name: student.name
    };
    do_insert(&student);
    Some(student)
}

// helper method to perform insert.
fn do_insert(message: &Student) {
    STORAGE.with(|service| service.borrow_mut().insert(message.id, message.clone()));
}

#[ic_cdk::update]
fn update_student(id: u64, payload: StudentPayload) -> Result<Student, Error> {
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut student) => {
            student.age = payload.age;
            student.email = payload.email;
            student.hobby = payload.hobby;
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
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(message) => Ok(message),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a message with id={}. message not found.",
                id
            ),
        }),
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

// need this to generate candid
ic_cdk::export_candid!();
