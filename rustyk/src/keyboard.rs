use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use futures_util::task::AtomicWaker;
use futures_util::stream::{Stream, StreamExt};
use core::pin::Pin;
use core::task::{Context, Poll};
use core::fmt::Write;
use crate::println;
use crate::console::Console;
use alloc::sync::Arc;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(err) = queue.push(scancode) {
            println!("WARNING: scanocde queue error: {}", err);
        }
        WAKER.wake();
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(ScancodeSet1::new(),
            layouts::Us104Key, HandleControl::Ignore)
        );
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE.try_get().expect("Scancode queue not initialized");

        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                println!("Scancode: {}", scancode);
                return Poll::Ready(Some(scancode));
            }
            None => {
                return Poll::Pending;
            }
        } 
    }
}

pub async fn print_keypresses(console: Arc<Mutex<Console>>) {
    
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(ScancodeSet1::new(),
        layouts::Us104Key, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => write!(console.lock(), "{}", character).unwrap(),
                    DecodedKey::RawKey(key) => write!(console.lock(), "{:?}", key).unwrap(),
                }
            }
        }
    }
}