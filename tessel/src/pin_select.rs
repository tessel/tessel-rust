//! TODO: make this a macro
//! TODO: we make two expensive runtime checks, which is ensuring that the
//! possible set of pins is valid (this can be eliminated) and ensuring that the
//! tuple contains only non-duplicate values (which can't be eliminated). We
//! can probably get this down to a single SSE instruction, if we wanted...

use bit_set::BitSet;
use std::sync::{Arc, Mutex};
use Pin;
use protocol::PortSocket;

/// Pin tuple conversion for gpio:pins(..)
pub trait PinSelect<'a> {
    type Output;
    fn validate(&self, &BitSet<usize>) -> bool;
    fn select(&self, socket: Arc<Mutex<PortSocket>>) -> Self::Output;
}

impl<'a> PinSelect<'a> for usize {
    type Output = Pin<'a>;
    fn validate(&self, set: &BitSet<usize>) -> bool {
        set.contains(*self)
    }
    fn select<'b>(&self, socket: Arc<Mutex<PortSocket>>) -> Self::Output {
        Pin::new(*self, socket)
    }
}

impl<'a> PinSelect<'a> for (usize, usize) {
    type Output = (Pin<'a>, Pin<'a>);
    fn validate(&self, set: &BitSet<usize>) -> bool {
        set.contains(self.0)
            || set.contains(self.1)
    }
    fn select<'b>(&self, socket: Arc<Mutex<PortSocket>>) -> Self::Output {
        (Pin::new(self.0, socket.clone()),
            Pin::new(self.1, socket.clone()))
    }
}

impl<'a> PinSelect<'a> for (usize, usize, usize) {
    type Output = (Pin<'a>, Pin<'a>, Pin<'a>);
    fn validate(&self, set: &BitSet<usize>) -> bool {
        set.contains(self.0)
            || set.contains(self.1)
            || set.contains(self.2)
    }
    fn select<'b>(&self, socket: Arc<Mutex<PortSocket>>) -> Self::Output {
        (Pin::new(self.0, socket.clone()),
            Pin::new(self.1, socket.clone()),
            Pin::new(self.2, socket.clone()))
    }
}

impl<'a> PinSelect<'a> for (usize, usize, usize, usize) {
    type Output = (Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>);
    fn validate(&self, set: &BitSet<usize>) -> bool {
        set.contains(self.0)
            || set.contains(self.1)
            || set.contains(self.2)
            || set.contains(self.3)
    }
    fn select<'b>(&self, socket: Arc<Mutex<PortSocket>>) -> Self::Output {
        (Pin::new(self.0, socket.clone()),
            Pin::new(self.1, socket.clone()),
            Pin::new(self.2, socket.clone()),
            Pin::new(self.3, socket.clone()))
    }
}

impl<'a> PinSelect<'a> for (usize, usize, usize, usize, usize) {
    type Output = (Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>);
    fn validate(&self, set: &BitSet<usize>) -> bool {
        set.contains(self.0)
            || set.contains(self.1)
            || set.contains(self.2)
            || set.contains(self.3)
            || set.contains(self.4)
    }
    fn select<'b>(&self, socket: Arc<Mutex<PortSocket>>) -> Self::Output {
        (Pin::new(self.0, socket.clone()),
            Pin::new(self.1, socket.clone()),
            Pin::new(self.2, socket.clone()),
            Pin::new(self.3, socket.clone()),
            Pin::new(self.4, socket.clone()))
    }
}

impl<'a> PinSelect<'a> for (usize, usize, usize, usize, usize, usize) {
    type Output = (Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>);
    fn validate(&self, set: &BitSet<usize>) -> bool {
        set.contains(self.0)
            || set.contains(self.1)
            || set.contains(self.2)
            || set.contains(self.3)
            || set.contains(self.4)
            || set.contains(self.5)
    }
    fn select<'b>(&self, socket: Arc<Mutex<PortSocket>>) -> Self::Output {
        (Pin::new(self.0, socket.clone()),
            Pin::new(self.1, socket.clone()),
            Pin::new(self.2, socket.clone()),
            Pin::new(self.3, socket.clone()),
            Pin::new(self.4, socket.clone()),
            Pin::new(self.5, socket.clone()))
    }
}

impl<'a> PinSelect<'a> for (usize, usize, usize, usize, usize, usize, usize) {
    type Output = (Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>);
    fn validate(&self, set: &BitSet<usize>) -> bool {
        set.contains(self.0)
            || set.contains(self.1)
            || set.contains(self.2)
            || set.contains(self.3)
            || set.contains(self.4)
            || set.contains(self.5)
            || set.contains(self.6)
    }
    fn select<'b>(&self, socket: Arc<Mutex<PortSocket>>) -> Self::Output {
        (Pin::new(self.0, socket.clone()),
            Pin::new(self.1, socket.clone()),
            Pin::new(self.2, socket.clone()),
            Pin::new(self.3, socket.clone()),
            Pin::new(self.4, socket.clone()),
            Pin::new(self.5, socket.clone()),
            Pin::new(self.6, socket.clone()))
    }
}


impl<'a> PinSelect<'a> for (usize, usize, usize, usize, usize, usize, usize, usize) {
    type Output = (Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>, Pin<'a>);
    fn validate(&self, set: &BitSet<usize>) -> bool {
        set.contains(self.0)
            || set.contains(self.1)
            || set.contains(self.2)
            || set.contains(self.3)
            || set.contains(self.4)
            || set.contains(self.5)
            || set.contains(self.6)
            || set.contains(self.7)
    }
    fn select<'b>(&self, socket: Arc<Mutex<PortSocket>>) -> Self::Output {
        (Pin::new(self.0, socket.clone()),
            Pin::new(self.1, socket.clone()),
            Pin::new(self.2, socket.clone()),
            Pin::new(self.3, socket.clone()),
            Pin::new(self.4, socket.clone()),
            Pin::new(self.5, socket.clone()),
            Pin::new(self.6, socket.clone()),
            Pin::new(self.7, socket.clone()))
    }
}
