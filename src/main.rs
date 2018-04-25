#![feature(test)]
#![feature(drain_filter)]
extern crate test;
extern crate try_drain;
use try_drain::*;

fn main() {
    let mut a = (0..10).collect::<Vec<_>>();
    {
        let mut d = a.drain_builder().try_drain(|el| {
            // break out

            match *el >= 4 {
                true => Flag::Return,
                false => Flag::Yield,
            }

            //*el < 4
        }).into_iter();
        for element in d.by_ref()
        /*a.drain_builder().try_drain(|mut el| {
            // break out
            match *el.get_mut() > 3 {
                true => el.stop(),
                false => el.take(),
            }
        }) */ {

            println!("2nd: {}", element);
        }
        println!("EXTRA: {:?}", d.by_ref().next());

    }

    for element in a.drain_builder().try_drain(|el| if *el < 7 { Flag::Yield } else { Flag::Break }).into_iter()
    //for element in a.drain_builder().try_drain(|el| *el < 7)
        .take(3)
    {
        println!("{}", element);
    }

    println!("{:?}", a);
}

#[allow(unused)] const MAX_REGULAR: usize = 100_000;
#[allow(unused)] const MAX_STRING: usize = 100_000;


macro_rules! make_bench {
    ( $iv:expr ; $( $name:ident , $make_drain:expr),* ) => {
        $(
            #[bench]
            fn $name (b: &mut test::Bencher) {
                let mut vec = ($iv).collect::<Vec<_>>();
                let mut vec2 = Vec::with_capacity(vec.len());

                b.iter(|| {
                    vec2.extend( $make_drain(&mut vec) );
                    std::mem::swap(&mut vec, &mut vec2);
                })
            }
        )*
    };
}

make_bench! {
    (0..MAX_REGULAR);
    _0_drain, Vec::drain_builder
}

#[bench]
fn _0a_noop(b: &mut test::Bencher) {
    let mut vec = (0..MAX_REGULAR).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(MAX_REGULAR);
    b.iter(|| {
        vec2.append(&mut vec);
        std::mem::swap(&mut vec, &mut vec2);
    })
}
/*
#[bench]
fn _0_drain(b: &mut test::Bencher) {
    let mut vec = (0..MAX_REGULAR).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(MAX_REGULAR);
    b.iter(|| {
        vec2.extend(vec.drain(..));
        std::mem::swap(&mut vec, &mut vec2);
    })
}
*/

#[bench]
fn _0_try_drain(b: &mut test::Bencher) {
    let mut vec = (0..MAX_REGULAR).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(MAX_REGULAR);
    b.iter(|| {
        vec2.extend(vec.drain_builder().try_drain(|_| Flag::Yield));
        assert!(vec2.len() == MAX_REGULAR);
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _0_elem_try_drain(b: &mut test::Bencher) {
    let mut vec = (0..MAX_REGULAR).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(MAX_REGULAR);
    b.iter(|| {
        vec2.extend(vec.drain_builder().elem_try_drain(|_| Flag::Yield));
        assert!(vec2.len() == MAX_REGULAR);
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _0_elem_move_drain(b: &mut test::Bencher) {
    let mut vec = (0..MAX_REGULAR).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(MAX_REGULAR);
    b.iter(|| {
        vec2.extend(vec.drain_builder().elem_move_drain(|el| el.take()));
        assert!(vec2.len() == MAX_REGULAR);
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _0_drain_filter(b: &mut test::Bencher) {
    let mut vec = (0..MAX_REGULAR).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(MAX_REGULAR);
    b.iter(|| {
        vec2.extend(vec.drain_filter(|_| true));
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _0_stream_drain(b: &mut test::Bencher) {
    let mut vec = (0..MAX_REGULAR).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(MAX_REGULAR);
    b.iter(|| {
        {
            let mut dr = vec.drain_builder().stream_drain();
            while let Some(el) = dr.streaming_next() {
                //if (*el + 1) % 1_000_001 != 0 {
                vec2.push(el.take());
                //}
            };
        }

        std::mem::swap(&mut vec, &mut vec2);
    });
}

#[bench]
fn _1a_string_noop(b: &mut test::Bencher) {
    let mut vec = (0..MAX_STRING).map(|n| n.to_string()).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(MAX_STRING);
    b.iter(|| {
        vec2.append(&mut vec);
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _1_string_drain(b: &mut test::Bencher) {
    let mut vec = (0..MAX_STRING).map(|n| n.to_string()).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(MAX_STRING);
    b.iter(|| {
        vec2.extend(vec.drain(..));
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _1_string_drain_filter(b: &mut test::Bencher) {
    let mut vec = (0..MAX_STRING).map(|n| n.to_string()).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(MAX_STRING);
    b.iter(|| {
        vec2.extend(vec.drain_filter(|_| true));
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _1_string_try_drain(b: &mut test::Bencher) {
    let mut vec = (0..MAX_STRING).map(|n| n.to_string()).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(MAX_STRING);
    b.iter(|| {
        vec2.extend(vec.drain_builder().try_drain(|_| Flag::Yield));
        assert!(vec2.len() == MAX_STRING);
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _1_string_elem_try_drain(b: &mut test::Bencher) {
    let mut vec = (0..MAX_STRING).map(|n| n.to_string()).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(MAX_STRING);
    b.iter(|| {
        vec2.extend(vec.drain_builder().elem_try_drain(|_| Flag::Yield));
        assert!(vec2.len() == MAX_STRING);
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _1_string_elem_move_drain(b: &mut test::Bencher) {
    let mut vec = (0..MAX_STRING).map(|n| n.to_string()).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(MAX_STRING);
    b.iter(|| {
        vec2.extend(vec.drain_builder().elem_move_drain(|el| el.take()));
        assert!(vec2.len() == MAX_STRING);
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _1_string_stream_drain(b: &mut test::Bencher) {
    let mut vec = (0..MAX_STRING).map(|n| n.to_string()).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(1000);
    b.iter(|| {
        {
            let mut dr = vec.drain_builder().stream_drain();
            while let Some(el) = dr.streaming_next() {
                //if (1 + el.parse::<u64>().unwrap()) % 3000 != 0 {
                vec2.push(el.take());
                //}
            };
        }

        std::mem::swap(&mut vec, &mut vec2);
    })
}
