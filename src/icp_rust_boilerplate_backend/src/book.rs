use candid::{Decode, Encode};
use ic_stable_structures::{BoundedStorable, Storable};
use std::borrow::Cow;

use crate::{time, Error, BOOK_STORAGE, ID_COUNTER};

// Define the Book struct to represent a book in the system.
#[derive(candid::CandidType, Deserialize, Serialize, Clone)]
pub struct Book {
    pub id: u64,
    pub title: String,
    pub author: String,
    pub created_at: u64,
    pub updated_at: Option<u64>,
}

// Implement serialization and deserialization for Book.
impl Storable for Book {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Set limits for Book storage size and flexibility.
impl BoundedStorable for Book {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Define the payload structure for creating or updating a book.
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
pub struct BookPayload {
    title: String,
    author: String,
}

// Retrieve all books from the storage.
#[ic_cdk::query]
fn get_all_books() -> Result<Vec<Book>, Error> {
    let books = _get_all_books();
    Ok(books)
}

// Internal function to fetch all books as a vector.
fn _get_all_books() -> Vec<Book> {
    BOOK_STORAGE.with(|books| {
        books
            .borrow()
            .iter()
            .map(|(_, value)| value.clone())
            .collect()
    })
}

// Retrieve a specific book by its ID.
#[ic_cdk::query]
fn get_book(id: u64) -> Result<Book, Error> {
    match _get_book(&id) {
        Some(book) => Ok(book),
        None => Err(Error::NotFound {
            msg: format!("A book with id={} not found.", id),
        }),
    }
}

// Internal function to fetch a book by ID.
fn _get_book(id: &u64) -> Option<Book> {
    BOOK_STORAGE.with(|s| s.borrow().get(id))
}

// Add a new book to the registry.
#[ic_cdk::update]
fn add_book(payload: BookPayload) -> Result<Book, Error> {
    // Validate the input payload.
    if payload.title.trim().is_empty() || payload.author.trim().is_empty() {
        return Err(Error::InvalidInput {
            msg: "Title and author cannot be empty.".to_string(),
        });
    }

    // Generate a new unique ID for the book.
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    // Create the new book with the provided payload.
    let book = Book {
        id,
        title: payload.title,
        author: payload.author,
        created_at: time(),
        updated_at: None,
    };

    // Insert the book into storage.
    do_insert(&book);
    Ok(book)
}

// Helper function to insert a book into storage.
fn do_insert(book: &Book) {
    BOOK_STORAGE.with(|service| service.borrow_mut().insert(book.id, book.clone()));
}

// Update an existing book's details by ID.
#[ic_cdk::update]
fn update_book(id: u64, payload: BookPayload) -> Result<Book, Error> {
    // Validate the input payload.
    if payload.title.trim().is_empty() || payload.author.trim().is_empty() {
        return Err(Error::InvalidInput {
            msg: "Title and author cannot be empty.".to_string(),
        });
    }

    // Fetch the book from storage and update its details.
    match BOOK_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut book) => {
            book.title = payload.title;
            book.author = payload.author;
            book.updated_at = Some(time());
            do_insert(&book); // Save the updated book back to storage.
            Ok(book)
        }
        None => Err(Error::NotFound {
            msg: format!("Couldn't update a book with id={}. Book not found.", id),
        }),
    }
}

// Delete a book by ID from the registry.
#[ic_cdk::update]
fn delete_book(id: u64) -> Result<Book, Error> {
    // Remove the book from storage.
    match BOOK_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(book) => Ok(book),
        None => Err(Error::NotFound {
            msg: format!("Couldn't delete a book with id={}. Book not found.", id),
        }),
    }
}
