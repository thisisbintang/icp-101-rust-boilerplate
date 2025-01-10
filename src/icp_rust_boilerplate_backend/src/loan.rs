use candid::{Decode, Encode};
use ic_stable_structures::{BoundedStorable, Storable};
use std::borrow::Cow;

use crate::{time, Error, ID_COUNTER, LOAN_STORAGE};

// Define the Loan struct to represent a loan in the system.
#[derive(candid::CandidType, Deserialize, Serialize, Clone)]
pub struct Loan {
    id: u64,
    student_id: u64,
    book_id: u64,
    loan_date: u64,
    created_at: u64,
    updated_at: Option<u64>,
}

// Implement serialization and deserialization for Loan.
impl Storable for Loan {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Set limits for Loan storage size and flexibility.
impl BoundedStorable for Loan {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Define the payload structure for creating or updating a loan.
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
pub struct LoanPayload {
    pub student_id: u64,
    pub book_id: u64,
    pub loan_date: u64,
}

// Retrieve all loans from the storage.
#[ic_cdk::query]
fn get_all_loans() -> Result<Vec<Loan>, Error> {
    let loans = _get_all_loans();
    Ok(loans)
}

// Internal function to fetch all loans as a vector.
fn _get_all_loans() -> Vec<Loan> {
    LOAN_STORAGE.with(|loans| {
        loans
            .borrow()
            .iter()
            .map(|(_, value)| value.clone())
            .collect()
    })
}

// Retrieve a specific loan by its ID.
#[ic_cdk::query]
fn get_loan(id: u64) -> Result<Loan, Error> {
    match _get_loan(&id) {
        Some(loan) => Ok(loan),
        None => Err(Error::NotFound {
            msg: format!("A loan with id={} not found.", id),
        }),
    }
}

// Internal function to fetch a loan by ID.
fn _get_loan(id: &u64) -> Option<Loan> {
    LOAN_STORAGE.with(|s| s.borrow().get(id))
}

// Add a new loan to the registry.
#[ic_cdk::update]
fn add_loan(payload: LoanPayload) -> Result<Loan, Error> {
    // Validate the input payload.
    if payload.student_id == 0 || payload.book_id == 0 || payload.loan_date == 0 {
        return Err(Error::InvalidInput {
            msg: "Student ID, Book ID, and Loan Date must be non-zero.".to_string(),
        });
    }

    // Generate a new unique ID for the loan.
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    // Create the new loan with the provided payload.
    let loan = Loan {
        id,
        student_id: payload.student_id,
        book_id: payload.book_id,
        loan_date: payload.loan_date,
        created_at: time(),
        updated_at: None,
    };

    // Insert the loan into storage.
    do_insert(&loan);
    Ok(loan)
}

// Helper function to insert a loan into storage.
fn do_insert(loan: &Loan) {
    LOAN_STORAGE.with(|service| service.borrow_mut().insert(loan.id, loan.clone()));
}

// Update an existing loan's details by ID.
#[ic_cdk::update]
fn update_loan(id: u64, payload: LoanPayload) -> Result<Loan, Error> {
    // Validate the input payload.
    if payload.student_id == 0 || payload.book_id == 0 || payload.loan_date == 0 {
        return Err(Error::InvalidInput {
            msg: "Student ID, Book ID, and Loan Date must be non-zero.".to_string(),
        });
    }

    // Fetch the loan from storage and update its details.
    match LOAN_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut loan) => {
            loan.student_id = payload.student_id;
            loan.book_id = payload.book_id;
            loan.loan_date = payload.loan_date;
            loan.updated_at = Some(time());
            do_insert(&loan); // Save the updated loan back to storage.
            Ok(loan)
        }
        None => Err(Error::NotFound {
            msg: format!("Couldn't update a loan with id={}. Loan not found.", id),
        }),
    }
}

// Delete a loan by ID from the registry.
#[ic_cdk::update]
fn delete_loan(id: u64) -> Result<Loan, Error> {
    // Remove the loan from storage.
    match LOAN_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(loan) => Ok(loan),
        None => Err(Error::NotFound {
            msg: format!("Couldn't delete a loan with id={}. Loan not found.", id),
        }),
    }
}
