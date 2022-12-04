use crate::mc::Connection;

pub mod mc;

pub struct User {
    pub mc: Connection,
}
