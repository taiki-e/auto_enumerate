#![cfg_attr(feature = "generator_trait", feature(generator_trait))]
#![cfg_attr(feature = "fn_traits", feature(fn_traits, unboxed_closures))]
#![cfg_attr(feature = "trusted_len", feature(trusted_len))]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

use auto_enums::enum_derive;

#[test]
fn unfmt() {
    #[rustfmt::skip]
    #[enum_derive(Debug, Iterator)]
    enum Enum1<A, B,> {
        A(A),
        B(B)
    }

    #[rustfmt::skip]
    #[enum_derive(Iterator)]
    enum Enum2<> {
        A(::core::ops::Range<i32>),
        B(::core::ops::RangeInclusive<i32>),
    }
}

#[test]
fn stable() {
    #[enum_derive(
        Iterator,
        DoubleEndedIterator,
        ExactSizeIterator,
        FusedIterator,
        Extend,
        Debug,
        Display,
        fmt::Write,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Future
    )]
    enum Stable<A, B> {
        A(A),
        B(B),
    }

    #[cfg(feature = "ops")]
    #[enum_derive(Deref, DerefMut, Index, IndexMut, RangeBounds)]
    enum Ops<A, B> {
        A(A),
        B(B),
    }

    #[cfg(feature = "convert")]
    #[enum_derive(AsRef, AsMut)]
    enum Convert<A, B> {
        A(A),
        B(B),
    }

    #[cfg(feature = "fmt")]
    #[enum_derive(
        fmt::Binary,
        fmt::LowerExp,
        fmt::LowerHex,
        fmt::Octal,
        fmt::Pointer,
        fmt::UpperExp,
        fmt::UpperHex
    )]
    enum Fmt<A, B> {
        A(A),
        B(B),
    }

    #[cfg(feature = "transpose_methods")]
    #[enum_derive(Transpose)]
    enum Transpose<A, B> {
        A(A),
        B(B),
    }

    #[enum_derive(Iterator, Clone)]
    #[enum_derive(Extend, Copy)]
    enum Enum3<A, B> {
        A(A),
        B(B),
    }
}

#[cfg(feature = "std")]
#[test]
fn stable_std() {
    #[enum_derive(
        BufRead, Read, Seek, Write, Display, Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd,
        Ord, Hash
    )]
    enum Stable<A, B> {
        A(A),
        B(B),
    }
}

// nightly

#[cfg(feature = "generator_trait")]
#[test]
fn generator_trait() {
    #[enum_derive(Generator)]
    enum Enum1<A, B> {
        A(A),
        B(B),
    }
}

#[cfg(feature = "fn_traits")]
#[test]
fn fn_traits() {
    #[enum_derive(Fn, FnMut, FnOnce)]
    enum Enum1<A, B> {
        A(A),
        B(B),
    }
}

#[cfg(feature = "trusted_len")]
#[test]
fn trusted_len() {
    #[enum_derive(TrustedLen)]
    enum Enum1<A, B> {
        A(A),
        B(B),
    }
}
