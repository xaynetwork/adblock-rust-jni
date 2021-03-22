#![cfg(target_os = "android")]
#![allow(non_snake_case)]
#[macro_use]
extern crate log;
extern crate android_logger;

use adblock::blocker::BlockerResult;
use adblock::engine::Engine;
use android_logger::Config;
use jni::objects::{JObject, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use log::Level;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::prelude::*;

// use std::ptr;
// use lazy_static::lazy_static;
// use std::sync::Mutex;
// use once_cell::sync::OnceCell;

#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_search_adblock_Adblock_hello(
    env: JNIEnv,
    _: JObject,
    j_recipient: JString,
) -> jstring {
    let recipient = CString::from(CStr::from_ptr(
        env.get_string(j_recipient).unwrap().as_ptr(),
    ));

    let output = env
        .new_string("Hello ".to_owned() + recipient.to_str().unwrap())
        .unwrap();
    output.into_inner()
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_search_adblock_Adblock_match(
    env: JNIEnv,
    _: JObject,
    j_url: JString,
) -> bool {
    let path = CString::from(CStr::from_ptr(env.get_string(j_url).unwrap().as_ptr()));

    match_url(path.to_str().unwrap().to_owned())
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_search_adblock_Adblock_init(
    env: JNIEnv,
    _: JObject,
    j_path: JString,
) {
    android_logger::init_once(Config::default().with_min_level(Level::Debug));

    let path = CString::from(CStr::from_ptr(env.get_string(j_path).unwrap().as_ptr()))
        .to_str()
        .unwrap()
        .to_owned();
    debug!("Will try to init with {:?}", path);

    init(path);
}

static mut ENGINE: Option<Engine> = None;

fn match_url(url: String) -> bool {
    unsafe {
        if let Some(engine) = &ENGINE {
            let result = engine.check_network_urls(&url, "", "") as BlockerResult;
            return result.matched;
        }
    }
    false
}

fn init(filepath: String) {
    debug!("Deserializing engine");
    // let engine = if let Some(engine) = unsafe { engine.as_ref() } { engine } else { panic!("No engine found") };
    let mut engine = Engine::default();

    let mut file = match File::open(&filepath) {
        Err(why) => {
            error!("couldn't open {}: {}", filepath, why);
            panic!()
        }
        Ok(file) => file,
    };
    let mut serialized = Vec::<u8>::new();
    file.read_to_end(&mut serialized)
        .expect("Reading from serialization file failed");

    // let mut file1 = File::open("rs-de.dat").expect("Opening serialization file failed");
    // let mut serialized1 = Vec::<u8>::new();
    // file1.read_to_end(&mut serialized1).expect("Reading from serialization file failed");
    engine
        .deserialize(&serialized)
        .expect("Deserialization failed");
    // engine.deserialize(&serialized1).expect("Deserialization failed of 1 ");
    // engine = get_blocker_engine();
    engine.use_tags(&["twitter-embeds"]);

    unsafe {
        ENGINE = Some(engine);
    }
}
