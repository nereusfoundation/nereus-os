use framebuffer::color::{Color, INFO};
use qwertz::Qwertz;
use sync::spin::SpinLock;

use crate::{handle_scancode, print, println};
use core::marker::PhantomData;

pub(crate) static KEYBOARD: SpinLock<Keyboard<Qwertz>> = SpinLock::new(Keyboard::new());

pub(crate) mod qwertz;
const KEYBOARD_COLOR: Color = INFO;

#[derive(Debug)]
pub(crate) struct Keyboard<T>
where
    T: KeyboardType,
{
    is_left_shift: bool,
    is_right_shift: bool,
    _marker: PhantomData<T>,
}

impl<T> Keyboard<T>
where
    T: KeyboardType,
{
    const fn new() -> Self {
        Self {
            is_left_shift: false,
            is_right_shift: false,
            _marker: PhantomData,
        }
    }

    pub(crate) fn handle(&mut self, scancode: u8) {
        handle_scancode!(self, scancode, T,
            |ascii| {
                if ascii != '\0' {
                    print!(KEYBOARD_COLOR, "{}", ascii);
                }
            },
            T::LEFT_SHIFT => { self.is_left_shift = true; },
            T::LEFT_SHIFT + 0x80 => { self.is_left_shift = false; },
            T::RIGHT_SHIFT => { self.is_right_shift = true; },
            T::RIGHT_SHIFT + 0x80 => { self.is_right_shift = false; },
            T::ENTER => println!()
        );
    }
}

pub(crate) trait KeyboardType {
    const LEFT_SHIFT: u8;
    const RIGHT_SHIFT: u8;

    const ENTER: u8;

    const ASCII_TABLE: [char; 58];

    fn translate(scancode: u8, uppercase: bool) -> char;
}

#[macro_export]
macro_rules! handle_scancode {
    ($self:ident, $scancode:ident, $type:ty, $default_action:expr, $($key:expr => $action:stmt), *) => {
        // specific action for specific key
        $(
            if $scancode == $key {
                $action
                return;
            }
        )*
        // default action
        {
            let ascii = <$type>::translate($scancode, $self.is_left_shift || $self.is_right_shift);
            $default_action(ascii);
        }
    }
}
