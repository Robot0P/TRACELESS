//! Secure memory handling module
//!
//! Uses the zeroize crate to securely clear sensitive data from memory
//! after use, preventing memory forensics from recovering secrets.

use zeroize::{Zeroize, ZeroizeOnDrop};
use std::ops::{Deref, DerefMut};

/// A wrapper for sensitive strings that auto-zeroizes on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureString {
    inner: String,
}

impl SecureString {
    /// Create a new secure string
    pub fn new(s: String) -> Self {
        Self { inner: s }
    }

    /// Create from a string slice
    pub fn from_str(s: &str) -> Self {
        Self { inner: s.to_string() }
    }

    /// Get the inner string (be careful with this!)
    pub fn expose(&self) -> &str {
        &self.inner
    }

    /// Consume and expose the inner string
    pub fn expose_owned(self) -> String {
        let s = self.inner.clone();
        // self will be dropped and zeroized
        s
    }
}

impl Default for SecureString {
    fn default() -> Self {
        Self { inner: String::new() }
    }
}

impl std::fmt::Debug for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

/// A wrapper for sensitive byte vectors that auto-zeroizes on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureBytes {
    inner: Vec<u8>,
}

impl SecureBytes {
    /// Create new secure bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { inner: bytes }
    }

    /// Create from a slice
    pub fn from_slice(bytes: &[u8]) -> Self {
        Self { inner: bytes.to_vec() }
    }

    /// Create with a specific capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self { inner: Vec::with_capacity(capacity) }
    }

    /// Get the inner bytes (be careful with this!)
    pub fn expose(&self) -> &[u8] {
        &self.inner
    }

    /// Get mutable access to inner bytes
    pub fn expose_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for SecureBytes {
    fn default() -> Self {
        Self { inner: Vec::new() }
    }
}

impl std::fmt::Debug for SecureBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED BYTES]")
    }
}

/// A fixed-size secure buffer that auto-zeroizes on drop
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecureBuffer<const N: usize> {
    inner: [u8; N],
    len: usize,
}

impl<const N: usize> SecureBuffer<N> {
    /// Create a new zeroed secure buffer
    pub fn new() -> Self {
        Self {
            inner: [0u8; N],
            len: 0,
        }
    }

    /// Create from a slice (truncates if too long)
    pub fn from_slice(bytes: &[u8]) -> Self {
        let mut buffer = Self::new();
        let copy_len = bytes.len().min(N);
        buffer.inner[..copy_len].copy_from_slice(&bytes[..copy_len]);
        buffer.len = copy_len;
        buffer
    }

    /// Get the used portion
    pub fn as_slice(&self) -> &[u8] {
        &self.inner[..self.len]
    }

    /// Get mutable access
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.inner[..self.len]
    }

    /// Set the used length
    pub fn set_len(&mut self, len: usize) {
        self.len = len.min(N);
    }

    /// Get the capacity
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Get the current length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<const N: usize> Default for SecureBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Clone for SecureBuffer<N> {
    fn clone(&self) -> Self {
        let mut new_buffer = Self::new();
        new_buffer.inner.copy_from_slice(&self.inner);
        new_buffer.len = self.len;
        new_buffer
    }
}

impl<const N: usize> std::fmt::Debug for SecureBuffer<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED BUFFER]")
    }
}

/// Guard for temporarily exposing sensitive data
/// Automatically zeroizes when dropped
pub struct SensitiveGuard<'a, T: Zeroize> {
    data: &'a mut T,
}

impl<'a, T: Zeroize> SensitiveGuard<'a, T> {
    pub fn new(data: &'a mut T) -> Self {
        Self { data }
    }
}

impl<T: Zeroize> Deref for SensitiveGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<T: Zeroize> DerefMut for SensitiveGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<T: Zeroize> Drop for SensitiveGuard<'_, T> {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

/// Securely generate random bytes
pub fn secure_random_bytes(len: usize) -> SecureBytes {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..len).map(|_| rng.gen()).collect();
    SecureBytes::new(bytes)
}

/// Securely clear a memory region
pub fn secure_clear(data: &mut [u8]) {
    data.zeroize();
}

/// Securely clear a string
pub fn secure_clear_string(s: &mut String) {
    s.zeroize();
}

/// Context for handling sensitive operations
/// Tracks allocations that need secure cleanup
pub struct SecureContext {
    strings: Vec<String>,
    bytes: Vec<Vec<u8>>,
}

impl SecureContext {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            bytes: Vec::new(),
        }
    }

    /// Allocate a secure string within this context
    pub fn alloc_string(&mut self, s: String) -> &str {
        self.strings.push(s);
        self.strings.last().unwrap()
    }

    /// Allocate secure bytes within this context
    pub fn alloc_bytes(&mut self, b: Vec<u8>) -> &[u8] {
        self.bytes.push(b);
        self.bytes.last().unwrap()
    }
}

impl Default for SecureContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SecureContext {
    fn drop(&mut self) {
        // Securely clear all tracked allocations
        for s in &mut self.strings {
            s.zeroize();
        }
        for b in &mut self.bytes {
            b.zeroize();
        }
    }
}

/// Macro for creating a secure scope that auto-clears variables
#[macro_export]
macro_rules! secure_scope {
    ($($var:ident),+ => $body:block) => {{
        let result = $body;
        $(
            $var.zeroize();
        )+
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_string() {
        let secret = SecureString::new("password123".to_string());
        assert_eq!(secret.expose(), "password123");
        assert_eq!(format!("{:?}", secret), "[REDACTED]");
    }

    #[test]
    fn test_secure_bytes() {
        let secret = SecureBytes::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(secret.expose(), &[1, 2, 3, 4, 5]);
        assert_eq!(secret.len(), 5);
        assert!(!secret.is_empty());
    }

    #[test]
    fn test_secure_buffer() {
        let buffer: SecureBuffer<32> = SecureBuffer::from_slice(b"hello");
        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.as_slice(), b"hello");
        assert_eq!(buffer.capacity(), 32);
    }

    #[test]
    fn test_secure_random_bytes() {
        let bytes = secure_random_bytes(32);
        assert_eq!(bytes.len(), 32);
        // Verify randomness (not all zeros)
        assert!(bytes.expose().iter().any(|&b| b != 0));
    }

    #[test]
    fn test_secure_context() {
        let mut ctx = SecureContext::new();
        let s = ctx.alloc_string("secret".to_string());
        assert_eq!(s, "secret");
        // Context will zeroize on drop
    }

    #[test]
    fn test_secure_clear() {
        let mut data = vec![1, 2, 3, 4, 5];
        secure_clear(&mut data);
        assert!(data.iter().all(|&b| b == 0));
    }
}
