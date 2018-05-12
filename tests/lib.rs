#![feature(test)]
#![feature(drain_filter)]
extern crate test;
extern crate try_drain;
use try_drain::*;
use try_drain::flag::Flag;

#[allow(unused)] const MAX: usize = 100_000;

struct DropBomb;

impl std::ops::Drop for DropBomb {
    fn drop(&mut self) {
        panic!("I dare you, I double dare you, motherfucker")
    }
}

#[allow(unused)]
fn fixate_types<T, F: FnOnce(&mut Vec<T>, &mut Vec<T>)>(f: F) -> F { f }

macro_rules! drain_all_test {
    ( $iv:expr, $type:ty ; $( $name:ident , $swap_via_drain:expr),* ) => {
        $(
            #[test]
            fn $name () {
                let swapper = fixate_types::<$type, _>($swap_via_drain);
                let mut vec = ($iv).collect::<Vec<_>>();
                let mut vec2 = Vec::with_capacity(vec.len());

                swapper(&mut vec, &mut vec2);
                for bomb in vec2 {
                    std::mem::forget(bomb);
                }
            }
        )*
    }
}

drain_all_test! {
    (0..MAX).map(|_| DropBomb), DropBomb;
    _0a_noop, |v1, v2| v2.append(v1),
    _0b_bomb_drain, |v1, v2| v2.extend(v1.drain(..)),
    _0c_bomb_drain_filter, |v1, v2| v2.extend(v1.drain_filter(|_| true)),
    _0d_bomb_try_drain, |v1, v2| v2.extend(v1.drain_builder().try_drain(|_| Flag::Yield)),
    _0d_bomb_elem_try_drain, |v1, v2| v2.extend(v1.drain_builder().elem_try_drain(|_| Flag::Yield)),
    _0d_bomb_elem_move_drain, |v1, v2| v2.extend(v1.drain_builder().elem_move_drain(|el| el.take())),
    _0e_bomb_stream_drain, |v1, v2| {
        let mut dr = v1.drain_builder().stream_drain();
            while let Some(el) = dr.streaming_next() {
                v2.push(el.take());
        };
    },
    _0f_bomb_cursor_iter, |v1, v2| v2.extend(v1.cursor().iter(|c| c.take())),
    _0f_bomb_cursor, |v1, v2| {
        let mut cursor = v1.cursor();
        while let Some(el) = cursor.take() {
            v2.push(el);
        }
    }
}

#[allow(unused)]
fn fixate_types_exhaust<T, F: FnOnce(&mut Vec<T>)>(f: F) -> F { f }

macro_rules! exhaust_test {
    ( $iv:expr, $type:ty ; $( $name:ident , $exhaust_via_drain:expr),* ) => {
        $(
            #[test]
            fn $name () {
                let exhauster = fixate_types_exhaust::<$type, _>($exhaust_via_drain);
                let mut vec = ($iv).collect::<Vec<_>>();
                exhauster(&mut vec);
                assert!(vec.is_empty());
            }
        )*
    };
}

exhaust_test! {
    (0..MAX).map(|n| n.to_string()), String;
    _1a_noop, |v| v.clear(),
    _1b_exhaust_string_drain, |v| { v.drain(..); },
    _1c_exhaust_string_drain_filter, |v| { v.drain_filter(|_| true); },
    _1d_exhaust_string_try_drain, |v| { v.drain_builder().try_drain(|_| Flag::Yield).into_iter().for_each(drop); },
    _1d_exhaust_string_elem_try_drain, |v| v.drain_builder().elem_try_drain(|_| Flag::Yield).for_each(drop),
    _1d_exhaust_string_elem_move_drain, |v| v.drain_builder().elem_move_drain(|el| el.take()).for_each(drop),
    _1e_exhaust_string_stream_drain, |v| { let mut dr = v.drain_builder().stream_drain();
            while let Some(el) = dr.streaming_next() {
                el.take();
            }},
    _1f_exhaust_string_cursor, |v| { let mut cursor = v.cursor(); while cursor.delete() {} },
    _1f_exhaust_string_cursor_iter, |v| v.cursor().iter(|c| c.take()).for_each(drop),
    _1f_exhaust_string_cursor_clear, |v| v.cursor().clear()
}
