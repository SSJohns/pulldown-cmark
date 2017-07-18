// Copyright 2015 Google Inc. All rights reserved.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! Pull parser for commonmark.

// When compiled for the rustc compiler itself we want to make sure that this is
// an unstable crate.
#![cfg_attr(rustbuild, feature(staged_api, rustc_private))]
#![cfg_attr(rustbuild, unstable(feature = "rustc_private", issue = "27812"))]

pub mod html;
pub mod rtjson;

#[macro_use]
extern crate bitflags;

mod passes;
mod parse;
mod scanners;
mod entities;
mod escape;
mod puncttable;
mod utils;

pub use passes::Parser;
pub use parse::{Event, Tag, Options, OPTION_ENABLE_TABLES, OPTION_ENABLE_FOOTNOTES};

extern crate libc;

use libc::c_char;
use std::ffi::{CStr, CString};

#[no_mangle]
pub extern fn html(s: *const c_char) -> CString {
    let c_str = unsafe {
        assert!(!s.is_null());

        CStr::from_ptr(s)
    };

    let r_str = c_str.to_str().unwrap();
    let mut s = String::with_capacity(r_str.len() * 3 / 2);
    let p = Parser::new_ext(r_str, Options::empty());
    html::push_html(&mut s, p);
    let c_str = CString::new(s).unwrap();
    c_str
}

#[no_mangle]
pub extern fn free_html(c: *mut c_char) {
    unsafe {
        CString::from_raw(c);
    }
}

// rtjson

#[no_mangle]
pub extern fn rtjson(s: *const c_char) -> CString {
    let c_str = unsafe {
        assert!(!s.is_null());

        CStr::from_ptr(s)
    };

    let r_str = c_str.to_str().unwrap();
    let mut s = String::with_capacity(r_str.len() * 3 / 2);
    let p = Parser::new_ext(r_str, Options::empty());
    rtjson::push_json(&mut s, p);
    let c_str = CString::new(s).unwrap();
    c_str
}

#[no_mangle]
pub extern fn free_html(c: *mut c_char) {
    unsafe {
        CString::from_raw(c);
    }
}