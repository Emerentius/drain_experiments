#![feature(test)]
#![feature(drain_filter)]
extern crate test;
extern crate try_drain;
use try_drain::*;

fn main() {
    let mut a = (0..10).collect::<Vec<_>>();
    {
        let mut d = a.try_drain(|mut el| {
            // break out

            match *el >= 4 {
                true => Flag::Return,
                false => Flag::Yield,
            }

            //*el < 4
        });
        for element in d.by_ref()
        /*a.try_drain(|mut el| {
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

    for element in a.try_drain(|el| if *el < 7 { Flag::Yield } else { Flag::Break })
    //for element in a.try_drain(|el| *el < 7)
        .take(3)
    {
        println!("{}", element);
    }

    println!("{:?}", a);
}

trait IteratorExt {
    fn exhausting(self) -> Exhausting<Self> where Self: Sized + Iterator;
}

impl<T: Iterator> IteratorExt for T {
    fn exhausting(self) -> Exhausting<Self>
    where
        Self: Sized
    {
        Exhausting {
            iter: self
        }
    }
}

struct Exhausting<T: Iterator> {
    iter: T
}

impl<T: Iterator> std::ops::Drop for Exhausting<T> {
    fn drop(&mut self) {
        for _ in self {}
    }
}

impl<T: Iterator> Iterator for Exhausting<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[bench]
fn _0a_noop(b: &mut test::Bencher) {
    let mut vec = (0..100_000).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(100_000);
    b.iter(|| {
        vec2.append(&mut vec);
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _0_drain(b: &mut test::Bencher) {
    let mut vec = (0..100_000).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(100_000);
    b.iter(|| {
        vec2.extend(vec.drain(..));
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _0_try_drain(b: &mut test::Bencher) {
    let mut vec = (0..100_000).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(100_000);
    b.iter(|| {
        vec2.extend(vec.try_drain(|_| Flag::Yield));
        assert!(vec2.len() == 100_000);
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _0_drain_filter(b: &mut test::Bencher) {
    let mut vec = (0..100_000).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(100_000);
    b.iter(|| {
        vec2.extend(vec.drain_filter(|_| true));
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _1a_string_noop(b: &mut test::Bencher) {
    let mut vec = (0..100_000).map(|n| n.to_string()).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(100_000);
    b.iter(|| {
        vec2.append(&mut vec);
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _1_string_drain(b: &mut test::Bencher) {
    let mut vec = (0..100_000).map(|n| n.to_string()).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(100_000);
    b.iter(|| {
        vec2.extend(vec.drain(..));
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _1_string_drain_filter(b: &mut test::Bencher) {
    let mut vec = (0..100_000).map(|n| n.to_string()).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(100_000);
    b.iter(|| {
        vec2.extend(vec.drain_filter(|_| true));
        std::mem::swap(&mut vec, &mut vec2);
    })
}

#[bench]
fn _1_string_try_drain(b: &mut test::Bencher) {
    let mut vec = (0..100_000).map(|n| n.to_string()).collect::<Vec<_>>();
    let mut vec2 = Vec::with_capacity(100_000);
    b.iter(|| {
        vec2.extend(vec.try_drain(|_| Flag::Yield));
        assert!(vec2.len() == 100_000);
        std::mem::swap(&mut vec, &mut vec2);
    })
}
