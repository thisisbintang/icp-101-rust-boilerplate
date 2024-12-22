use candid::{Decode, Encode};
use ic_stable_structures::{BoundedStorable, Storable};
use std::borrow::Cow;

use crate::{time, Error, BOOK_STORAGE, ID_COUNTER};

#[derive(candid::CandidType, Deserialize, Serialize, Clone)]
pub struct Book {
    pub id: u64,
    pub title: String,
    pub author: String,
    pub created_at: u64,
    pub updated_at: Option<u64>,
}

impl Storable for Book {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Book {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
pub struct BookPayload {
    title: String,
    author: String,
}

#[ic_cdk::query]
fn get_all_books() -> Result<Vec<Book>, Error> {
    let books = _get_all_books();

    Ok(books)
}

fn _get_all_books() -> Vec<Book> {
    BOOK_STORAGE.with(|books| {
        books
            .borrow()
            .iter()
            .map(|(_, value)| value.clone())
            .collect()
    })
}

#[ic_cdk::query]
fn get_book(id: u64) -> Result<Book, Error> {
    match _get_book(&id) {
        Some(book) => Ok(book),
        None => Err(Error::NotFound {
            msg: format!("a book with id={} not found", id),
        }),
    }
}

fn _get_book(id: &u64) -> Option<Book> {
    BOOK_STORAGE.with(|s| s.borrow().get(id))
}

#[ic_cdk::update]
fn add_book(book: BookPayload) -> Option<Book> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let book = Book {
        id,
        created_at: time(),
        updated_at: None,
        author: book.author,
        title: book.title,
    };
    do_insert(&book);
    Some(book)
}

// helper method to perform insert.
fn do_insert(book: &Book) {
    BOOK_STORAGE.with(|service| service.borrow_mut().insert(book.id, book.clone()));
}

#[ic_cdk::update]
fn update_book(id: u64, payload: BookPayload) -> Result<Book, Error> {
    match BOOK_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut book) => {
            book.author = payload.author;
            book.title = payload.title;
            book.updated_at = Some(time());
            do_insert(&book);
            Ok(book)
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update a book with id={}. book not found", id),
        }),
    }
}

#[ic_cdk::update]
fn delete_book(id: u64) -> Result<Book, Error> {
    match BOOK_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(book) => Ok(book),
        None => Err(Error::NotFound {
            msg: format!("couldn't delete a book with id={}. book not found.", id),
        }),
    }
}
