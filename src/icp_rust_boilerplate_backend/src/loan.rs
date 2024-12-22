use candid::{Decode, Encode};
use ic_stable_structures::{BoundedStorable, Storable};
use std::borrow::Cow;

use crate::{time, Error, ID_COUNTER, LOAN_STORAGE};

#[derive(candid::CandidType, Deserialize, Serialize, Clone)]
pub struct Loan {
    id: u64,
    student_id: u64,
    book_id: u64,
    loan_date: u64,
    created_at: u64,
    updated_at: Option<u64>,
}

impl Storable for Loan {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Loan {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
pub struct LoanPayload {
    pub student_id: u64,
    pub book_id: u64,
    pub loan_date: u64,
}

#[ic_cdk::query]
fn get_all_loans() -> Result<Vec<Loan>, Error> {
    let loans = _get_all_loans();

    Ok(loans)
}

fn _get_all_loans() -> Vec<Loan> {
    LOAN_STORAGE.with(|loans| {
        loans
            .borrow()
            .iter()
            .map(|(_, value)| value.clone())
            .collect()
    })
}

#[ic_cdk::query]
fn get_loan(id: u64) -> Result<Loan, Error> {
    match _get_loan(&id) {
        Some(loan) => Ok(loan),
        None => Err(Error::NotFound {
            msg: format!("a loan with id={} not found", id),
        }),
    }
}

fn _get_loan(id: &u64) -> Option<Loan> {
    LOAN_STORAGE.with(|s| s.borrow().get(id))
}

#[ic_cdk::update]
fn add_loan(loan: LoanPayload) -> Option<Loan> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let loan = Loan {
        id,
        created_at: time(),
        updated_at: None,
        book_id: loan.book_id,
        loan_date: loan.book_id,
        student_id: loan.student_id,
    };
    do_insert(&loan);
    Some(loan)
}

// helper method to perform insert.
fn do_insert(loan: &Loan) {
    LOAN_STORAGE.with(|service| service.borrow_mut().insert(loan.id, loan.clone()));
}

#[ic_cdk::update]
fn update_loan(id: u64, payload: LoanPayload) -> Result<Loan, Error> {
    match LOAN_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut loan) => {
            loan.book_id = payload.book_id;
            loan.loan_date = payload.loan_date;
            loan.student_id = payload.student_id;
            loan.updated_at = Some(time());
            do_insert(&loan);
            Ok(loan)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update a loan with id={}. loan not found",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn delete_loan(id: u64) -> Result<Loan, Error> {
    match LOAN_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(loan) => Ok(loan),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a loan with id={}. loan not found.",
                id
            ),
        }),
    }
}
