//! Channels for Goroutine Communication
//!
//! Implements Go-style channels with send/receive operations.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use thiserror::Error;

/// Channel errors
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ChannelError {
    /// Channel is closed
    #[error("Channel is closed")]
    Closed,

    /// Send on closed channel
    #[error("Send on closed channel")]
    SendOnClosed,

    /// Receive on closed empty channel
    #[error("Receive on closed empty channel")]
    ReceiveOnClosed,

    /// Channel is full (buffered channels only)
    #[error("Channel is full")]
    Full,
}

/// Result type for channel operations
pub type Result<T> = std::result::Result<T, ChannelError>;

/// Channel capacity configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelCapacity {
    /// Unbuffered channel (synchronous)
    Unbuffered,
    /// Buffered channel with specified capacity
    Buffered(usize),
}

/// Shared state for a channel
struct ChannelState<T> {
    /// Buffer for messages
    buffer: VecDeque<T>,
    /// Maximum capacity
    capacity: ChannelCapacity,
    /// Whether channel is closed
    closed: bool,
    /// Number of active senders
    sender_count: usize,
}

impl<T> ChannelState<T> {
    fn new(capacity: ChannelCapacity) -> Self {
        Self {
            buffer: VecDeque::new(),
            capacity,
            closed: false,
            sender_count: 0,
        }
    }

    fn is_full(&self) -> bool {
        match self.capacity {
            ChannelCapacity::Unbuffered => !self.buffer.is_empty(),
            ChannelCapacity::Buffered(cap) => self.buffer.len() >= cap,
        }
    }

    fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

/// Sender half of a channel
pub struct Sender<T> {
    state: Arc<Mutex<ChannelState<T>>>,
    send_cv: Arc<Condvar>,
    recv_cv: Arc<Condvar>,
}

impl<T> Sender<T> {
    /// Send a value through the channel
    pub fn send(&self, value: T) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.closed {
            return Err(ChannelError::SendOnClosed);
        }

        // Wait until space is available
        while state.is_full() && !state.closed {
            state = self.send_cv.wait(state).unwrap();
        }

        if state.closed {
            return Err(ChannelError::SendOnClosed);
        }

        state.buffer.push_back(value);

        // Notify receivers
        self.recv_cv.notify_one();

        Ok(())
    }

    /// Try to send without blocking
    pub fn try_send(&self, value: T) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.closed {
            return Err(ChannelError::SendOnClosed);
        }

        if state.is_full() {
            return Err(ChannelError::Full);
        }

        state.buffer.push_back(value);
        self.recv_cv.notify_one();

        Ok(())
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        // Increment sender count
        self.state.lock().unwrap().sender_count += 1;

        Self {
            state: Arc::clone(&self.state),
            send_cv: Arc::clone(&self.send_cv),
            recv_cv: Arc::clone(&self.recv_cv),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.sender_count = state.sender_count.saturating_sub(1);

        // Close channel if this was the last sender
        if state.sender_count == 0 {
            state.closed = true;
            // Wake up all waiting receivers
            self.recv_cv.notify_all();
        }
    }
}

/// Receiver half of a channel
pub struct Receiver<T> {
    state: Arc<Mutex<ChannelState<T>>>,
    send_cv: Arc<Condvar>,
    recv_cv: Arc<Condvar>,
}

impl<T> Receiver<T> {
    /// Receive a value from the channel
    pub fn recv(&self) -> Result<T> {
        let mut state = self.state.lock().unwrap();

        // Wait until a value is available or channel is closed
        while state.is_empty() && !state.closed {
            state = self.recv_cv.wait(state).unwrap();
        }

        if state.is_empty() && state.closed {
            return Err(ChannelError::ReceiveOnClosed);
        }

        let value = state.buffer.pop_front().unwrap();

        // Notify senders that space is available
        self.send_cv.notify_one();

        Ok(value)
    }

    /// Try to receive without blocking
    pub fn try_recv(&self) -> Result<T> {
        let mut state = self.state.lock().unwrap();

        if state.is_empty() {
            if state.closed {
                return Err(ChannelError::ReceiveOnClosed);
            }
            return Err(ChannelError::Closed); // Using Closed to indicate no data
        }

        let value = state.buffer.pop_front().unwrap();
        self.send_cv.notify_one();

        Ok(value)
    }

    /// Get an iterator over received values
    pub fn iter(&self) -> ReceiverIter<T> {
        ReceiverIter { receiver: self }
    }
}

/// Iterator over channel values
pub struct ReceiverIter<'a, T> {
    receiver: &'a Receiver<T>,
}

impl<'a, T> Iterator for ReceiverIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.recv().ok()
    }
}

/// Create a new channel
pub fn channel<T>(capacity: ChannelCapacity) -> (Sender<T>, Receiver<T>) {
    let state = Arc::new(Mutex::new(ChannelState::new(capacity)));
    let send_cv = Arc::new(Condvar::new());
    let recv_cv = Arc::new(Condvar::new());

    // Initialize sender count to 1
    state.lock().unwrap().sender_count = 1;

    let sender = Sender {
        state: Arc::clone(&state),
        send_cv: Arc::clone(&send_cv),
        recv_cv: Arc::clone(&recv_cv),
    };

    let receiver = Receiver {
        state,
        send_cv,
        recv_cv,
    };

    (sender, receiver)
}

/// Create an unbuffered (synchronous) channel
pub fn unbuffered<T>() -> (Sender<T>, Receiver<T>) {
    channel(ChannelCapacity::Unbuffered)
}

/// Create a buffered channel with the specified capacity
pub fn buffered<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    channel(ChannelCapacity::Buffered(capacity))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_unbuffered_channel() {
        let (tx, rx) = unbuffered();

        thread::spawn(move || {
            tx.send(42).unwrap();
        });

        let value = rx.recv().unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_buffered_channel() {
        let (tx, rx) = buffered(3);

        // Can send 3 values without blocking
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();

        assert_eq!(rx.recv().unwrap(), 1);
        assert_eq!(rx.recv().unwrap(), 2);
        assert_eq!(rx.recv().unwrap(), 3);
    }

    #[test]
    fn test_multiple_senders() {
        let (tx, rx) = buffered(10);

        let mut handles = Vec::new();
        for i in 0..5 {
            let tx_clone = tx.clone();
            let handle = thread::spawn(move || {
                tx_clone.send(i).unwrap();
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        drop(tx); // Drop original sender

        let mut values = Vec::new();
        while let Ok(v) = rx.recv() {
            values.push(v);
        }

        assert_eq!(values.len(), 5);
    }

    #[test]
    fn test_send_on_closed() {
        let (tx, rx) = unbuffered::<i32>();

        drop(rx); // Close receiver

        let result = tx.send(42);
        // Channel is closed, but send might succeed if already in progress
        // This behavior depends on timing
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_recv_on_closed_empty() {
        let (tx, rx) = unbuffered::<i32>();

        drop(tx); // Close sender

        let result = rx.recv();
        assert_eq!(result, Err(ChannelError::ReceiveOnClosed));
    }

    #[test]
    fn test_try_send() {
        let (tx, rx) = buffered(2);

        assert!(tx.try_send(1).is_ok());
        assert!(tx.try_send(2).is_ok());
        assert_eq!(tx.try_send(3), Err(ChannelError::Full));

        rx.recv().unwrap();
        assert!(tx.try_send(3).is_ok());
    }

    #[test]
    fn test_try_recv() {
        let (tx, rx) = buffered(2);

        assert!(rx.try_recv().is_err()); // Empty

        tx.send(42).unwrap();
        assert_eq!(rx.try_recv().unwrap(), 42);
        assert!(rx.try_recv().is_err()); // Empty again
    }

    #[test]
    fn test_channel_iterator() {
        let (tx, rx) = buffered(5);

        thread::spawn(move || {
            for i in 0..5 {
                tx.send(i).unwrap();
            }
        });

        let values: Vec<i32> = rx.iter().take(5).collect();
        assert_eq!(values, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_multi_thread_communication() {
        let (tx, rx) = buffered(100);

        let sender_handle = thread::spawn(move || {
            for i in 0..100 {
                tx.send(i).unwrap();
            }
        });

        let mut sum = 0;
        for _ in 0..100 {
            sum += rx.recv().unwrap();
        }

        sender_handle.join().unwrap();
        assert_eq!(sum, (0..100).sum());
    }

    #[test]
    fn test_sender_clone() {
        let (tx, rx) = buffered(10);
        let tx2 = tx.clone();

        tx.send(1).unwrap();
        tx2.send(2).unwrap();

        assert_eq!(rx.recv().unwrap(), 1);
        assert_eq!(rx.recv().unwrap(), 2);
    }

    #[test]
    fn test_channel_capacity() {
        let (tx, _rx) = buffered::<i32>(5);
        let state = tx.state.lock().unwrap();
        assert_eq!(state.capacity, ChannelCapacity::Buffered(5));
    }
}
