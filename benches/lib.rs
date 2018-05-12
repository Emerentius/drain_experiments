#![feature(test)]
#![feature(drain_filter)]
extern crate test;
extern crate try_drain;
use try_drain::*;
use try_drain::flag::Flag;

#[allow(unused)] const MAX_REGULAR: usize = 1_000_000;
#[allow(unused)] const MAX_STRING: usize = 100_000;

#[allow(unused)]
fn fixate_types<T, F: FnOnce(&mut Vec<T>, &mut Vec<T>)>(f: F) -> F { f }

macro_rules! drain_all_bench {
    ( $iv:expr, $type:ty ; $( $name:ident , $swap_via_drain:expr),* ) => {
        $(
            #[bench]
            fn $name (b: &mut test::Bencher) {
                let swapper = fixate_types::<$type, _>($swap_via_drain);
                let mut vec = ($iv).collect::<Vec<_>>();
                let mut vec2 = Vec::with_capacity(vec.len());

                b.iter(|| {
                    swapper(&mut vec, &mut vec2);
                    std::mem::swap(&mut vec, &mut vec2);
                })
            }
        )*
    };
}

drain_all_bench! {
    0..MAX_REGULAR, usize;
    _0a_noop, |v1, v2| v2.append(v1),
    _0b_drain, |v1, v2| v2.extend(v1.drain(..)),
    _0c_drain_filter, |v1, v2| v2.extend(v1.drain_filter(|_| true)),
    _0d_try_drain, |v1, v2| v2.extend(v1.drain_builder().try_drain(|_| Flag::Yield)),
    _0d_elem_try_drain, |v1, v2| v2.extend(v1.drain_builder().elem_try_drain(|_| Flag::Yield)),
    _0d_elem_move_drain, |v1, v2| v2.extend(v1.drain_builder().elem_move_drain(|el| el.take())),
    _0e_stream_drain, |v1, v2| {
        let mut dr = v1.drain_builder().stream_drain();
            while let Some(el) = dr.streaming_next() {
                v2.push(el.take());
        };
    },
    _0f_cursor_iter, |v1, v2| v2.extend(v1.cursor().iter(|c| c.take())),
    _0f_cursor, |v1, v2| {
        let mut cursor = v1.cursor();
        while let Some(el) = cursor.take() {
            v2.push(el);
        }
    }
}

drain_all_bench! {
    (0..MAX_STRING).map(|n| n.to_string()), String;
    _1a_noop, |v1, v2| v2.append(v1),
    _1b_string_drain, |v1, v2| v2.extend(v1.drain(..)),
    _1c_string_drain_filter, |v1, v2| v2.extend(v1.drain_filter(|_| true)),
    _1d_string_try_drain, |v1, v2| v2.extend(v1.drain_builder().try_drain(|_| Flag::Yield)),
    _1d_string_elem_try_drain, |v1, v2| v2.extend(v1.drain_builder().elem_try_drain(|_| Flag::Yield)),
    _1d_string_elem_move_drain, |v1, v2| v2.extend(v1.drain_builder().elem_move_drain(|el| el.take())),
    _1e_string_stream_drain, |v1, v2| {
        let mut dr = v1.drain_builder().stream_drain();
            while let Some(el) = dr.streaming_next() {
                v2.push(el.take());
        };
    },
    _1f_string_cursor_iter, |v1, v2| v2.extend(v1.cursor().iter(|c| c.take())),
    _1f_string_cursor, |v1, v2| {
        let mut cursor = v1.cursor();
        while let Some(el) = cursor.take() {
            v2.push(el);
        }
    }
}

#[allow(unused)]
fn fixate_types_exhaust<T, F: FnOnce(&mut Vec<T>)>(f: F) -> F { f }

macro_rules! exhaust_bench {
    ( $iv:expr, $type:ty ; $( $name:ident , $exhaust_via_drain:expr),* ) => {
        $(
            #[bench]
            fn $name (b: &mut test::Bencher) {
                let exhauster = fixate_types_exhaust::<$type, _>($exhaust_via_drain);
                let vec = ($iv).collect::<Vec<_>>();
                let mut vec2 = Vec::with_capacity(vec.len());

                b.iter(|| {
                    vec2.extend_from_slice(&vec);
                    exhauster(&mut vec2);
                })
            }
        )*
    };
}

exhaust_bench! {
    0..MAX_REGULAR, usize;
    _2a_noop, |v| v.clear(),
    _2b_exhaust_drain, |v| { v.drain(..); },
    _2c_exhaust_drain_filter, |v| { v.drain_filter(|_| true); },
    _2d_exhaust_try_drain, |v| { v.drain_builder().try_drain(|_| Flag::Yield).into_iter().for_each(drop); },
    _2d_exhaust_elem_try_drain, |v| v.drain_builder().elem_try_drain(|_| Flag::Yield).for_each(drop),
    _2d_exhaust_elem_move_drain, |v| v.drain_builder().elem_move_drain(|el| el.take()).for_each(drop),
    _2e_exhaust_stream_drain, |v| { let mut dr = v.drain_builder().stream_drain();
            while let Some(el) = dr.streaming_next() {
                el.take();
            }},

    // while let Some(_) = cursor.take() optimizes better but still suboptimally
    _2f_exhaust_cursor, |v| { let mut cursor = v.cursor(); while cursor.delete() {} },
    _2f_exhaust_cursor_iter, |v| v.cursor().iter(|c| c.take()).for_each(drop),
    _2f_exhaust_cursor_clear, |v| v.cursor().clear()
}

exhaust_bench! {
    (0..MAX_STRING).map(|n| n.to_string()), String;
    _3a_noop, |v| v.clear(),
    _3b_exhaust_string_drain, |v| { v.drain(..); },
    _3c_exhaust_string_drain_filter, |v| { v.drain_filter(|_| true); },
    _3d_exhaust_string_try_drain, |v| { v.drain_builder().try_drain(|_| Flag::Yield).into_iter().for_each(drop); },
    _3d_exhaust_string_elem_try_drain, |v| v.drain_builder().elem_try_drain(|_| Flag::Yield).for_each(drop),
    _3d_exhaust_string_elem_move_drain, |v| v.drain_builder().elem_move_drain(|el| el.take()).for_each(drop),
    _3e_exhaust_string_stream_drain, |v| { let mut dr = v.drain_builder().stream_drain();
            while let Some(el) = dr.streaming_next() {
                el.take();
            }},
    _3f_exhaust_string_cursor, |v| { let mut cursor = v.cursor(); while cursor.delete() {} },
    _3f_exhaust_string_cursor_iter, |v| v.cursor().iter(|c| c.take()).for_each(drop),
    _3f_exhaust_string_cursor_clear, |v| v.cursor().clear()
}
