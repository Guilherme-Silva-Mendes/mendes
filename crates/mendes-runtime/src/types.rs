//! Mendes runtime native types

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Deref};

/// Mendes String - wrapper over String with additional methods
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MendesString(pub String);

impl MendesString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, pattern: &str) -> bool {
        self.0.contains(pattern)
    }

    pub fn concat(&self, other: &MendesString) -> MendesString {
        MendesString(format!("{}{}", self.0, other.0))
    }
}

impl Add for MendesString {
    type Output = MendesString;

    fn add(self, other: MendesString) -> MendesString {
        MendesString(format!("{}{}", self.0, other.0))
    }
}

impl Add<&MendesString> for MendesString {
    type Output = MendesString;

    fn add(self, other: &MendesString) -> MendesString {
        MendesString(format!("{}{}", self.0, other.0))
    }
}

impl Add<&str> for MendesString {
    type Output = MendesString;

    fn add(self, other: &str) -> MendesString {
        MendesString(format!("{}{}", self.0, other))
    }
}

impl Deref for MendesString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for MendesString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for MendesString {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for MendesString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Mendes dynamic array
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MendesArray<T>(pub Vec<T>);

impl<T> MendesArray<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn push(&mut self, item: T) {
        self.0.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.0.get_mut(index)
    }
}

impl<T> Default for MendesArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for MendesArray<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromIterator<T> for MendesArray<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T> IntoIterator for MendesArray<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Mendes Result - Ok or Err
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MendesResult<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> MendesResult<T, E> {
    pub fn is_ok(&self) -> bool {
        matches!(self, MendesResult::Ok(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, MendesResult::Err(_))
    }

    pub fn ok(self) -> Option<T> {
        match self {
            MendesResult::Ok(v) => Some(v),
            MendesResult::Err(_) => None,
        }
    }

    pub fn err(self) -> Option<E> {
        match self {
            MendesResult::Ok(_) => None,
            MendesResult::Err(e) => Some(e),
        }
    }

    pub fn unwrap(self) -> T
    where
        E: fmt::Debug,
    {
        match self {
            MendesResult::Ok(v) => v,
            MendesResult::Err(e) => panic!("called unwrap on Err: {:?}", e),
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> MendesResult<U, E> {
        match self {
            MendesResult::Ok(v) => MendesResult::Ok(f(v)),
            MendesResult::Err(e) => MendesResult::Err(e),
        }
    }
}

impl<T, E> From<Result<T, E>> for MendesResult<T, E> {
    fn from(r: Result<T, E>) -> Self {
        match r {
            Ok(v) => MendesResult::Ok(v),
            Err(e) => MendesResult::Err(e),
        }
    }
}

impl<T, E> From<MendesResult<T, E>> for Result<T, E> {
    fn from(r: MendesResult<T, E>) -> Self {
        match r {
            MendesResult::Ok(v) => Ok(v),
            MendesResult::Err(e) => Err(e),
        }
    }
}

impl<T: fmt::Display, E: fmt::Display> fmt::Display for MendesResult<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MendesResult::Ok(v) => write!(f, "Ok({})", v),
            MendesResult::Err(e) => write!(f, "Err({})", e),
        }
    }
}

/// Mendes Option - Some or None
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MendesOption<T> {
    Some(T),
    None,
}

impl<T> MendesOption<T> {
    pub fn is_some(&self) -> bool {
        matches!(self, MendesOption::Some(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self, MendesOption::None)
    }

    pub fn unwrap(self) -> T {
        match self {
            MendesOption::Some(v) => v,
            MendesOption::None => panic!("called unwrap on None"),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            MendesOption::Some(v) => v,
            MendesOption::None => default,
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> MendesOption<U> {
        match self {
            MendesOption::Some(v) => MendesOption::Some(f(v)),
            MendesOption::None => MendesOption::None,
        }
    }
}

impl<T> Default for MendesOption<T> {
    fn default() -> Self {
        MendesOption::None
    }
}

impl<T> From<Option<T>> for MendesOption<T> {
    fn from(o: Option<T>) -> Self {
        match o {
            Some(v) => MendesOption::Some(v),
            None => MendesOption::None,
        }
    }
}

impl<T> From<MendesOption<T>> for Option<T> {
    fn from(o: MendesOption<T>) -> Self {
        match o {
            MendesOption::Some(v) => Some(v),
            MendesOption::None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mendes_string() {
        let s = MendesString::new("hello");
        assert_eq!(s.len(), 5);
        assert!(s.contains("ell"));

        let s2 = MendesString::new(" world");
        let concat = s.concat(&s2);
        assert_eq!(concat.0, "hello world");
    }

    #[test]
    fn test_mendes_array() {
        let mut arr: MendesArray<i64> = MendesArray::new();
        arr.push(1);
        arr.push(2);
        arr.push(3);

        assert_eq!(arr.len(), 3);
        assert_eq!(arr.get(0), Some(&1));
        assert_eq!(arr.pop(), Some(3));
    }

    #[test]
    fn test_mendes_result() {
        let ok: MendesResult<i64, String> = MendesResult::Ok(42);
        assert!(ok.is_ok());
        assert_eq!(ok.unwrap(), 42);

        let err: MendesResult<i64, String> = MendesResult::Err("error".to_string());
        assert!(err.is_err());
    }

    #[test]
    fn test_mendes_option() {
        let some: MendesOption<i64> = MendesOption::Some(42);
        assert!(some.is_some());
        assert_eq!(some.unwrap(), 42);

        let none: MendesOption<i64> = MendesOption::None;
        assert!(none.is_none());
        assert_eq!(none.unwrap_or(0), 0);
    }
}
