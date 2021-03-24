#![allow(non_snake_case)]
#[macro_use]
extern crate log;
extern crate android_logger;

use adblock::engine::Engine;
use android_logger::Config;
use jni::objects::{JObject, JString};
use jni::sys::{jlong, jbyte, jbyteArray};
use jni::JNIEnv;
use log::{Level, LevelFilter};
use std::fs::File;
use std::io::prelude::*;
use env_logger::Builder;
use adblock::resources::{Resource, ResourceType, MimeType};

macro_rules! throwAndPanic {
    ($env:expr,$message:expr)=>{
        {
            $env.throw_new("java/lang/RuntimeException", $message).expect("Can not throw exception in java env");
            error!("{}", $message);
            panic!("{}", $message)
        }
    }
}

static mut IS_INITIALIZED: bool = false;
const IS_MATCHED_MASK: i8 = 1;
const IS_IMPORTANT_MASK: i8 = 2;
const IS_EXCEPTION_MASK: i8 = 4;


unsafe fn check_init() {
    if !IS_INITIALIZED {
        if cfg!(android) {
            android_logger::init_once(Config::default().with_min_level(Level::Debug));
        } else {
            println!("init logger");
            let mut builder = Builder::new();
            builder.filter_level(LevelFilter::Debug);
            builder.init();
        };
        IS_INITIALIZED = true;
    }
}

unsafe fn unwrapString(env: &JNIEnv, jString: JString) -> String {
    let loadedRules: String = match env.get_string(jString) {
        Err(why) => {
            throwAndPanic!(env, format!("Could not convert JString to String: {}", why))
        }
        Ok(str) => str.into(),
    };
    loadedRules
}

unsafe fn unwrapEngine<'a>(env: &'a JNIEnv, enginePointer: i64) -> &'a mut Engine {
    let enginePointer = enginePointer as *mut Engine;
    let engine = if let Some(restored) = enginePointer.as_mut() { restored } else { throwAndPanic!(&env,  "Engine is not allocated anymore!") };
    engine
}


/// An external callback that receives a hostname and two out-parameters for start and end
/// position. The callback should fill the start and end positions with the start and end indices
/// of the domain part of the hostname.
// pub type DomainResolverCallback = unsafe extern "C" fn(*const c_char, *mut u32, *mut u32);

/// Passes a callback to the adblock library, allowing it to be used for domain resolution.
///
/// This is required to be able to use any adblocking functionality.
///
/// Returns true on success, false if a callback was already set previously.
// #[no_mangle]
// pub unsafe extern "C" fn set_domain_resolver(resolver: DomainResolverCallback) -> bool {
//     struct RemoteResolverImpl {
//         remote_callback: DomainResolverCallback,
//     }
//
//     impl adblock::url_parser::ResolvesDomain for RemoteResolverImpl {
//         fn get_host_domain(&self, host: &str) -> (usize, usize) {
//             let mut start: u32 = 0;
//             let mut end: u32 = 0;
//             let host_c_str = CString::new(host).expect("Error: CString::new()");
//             let remote_callback = self.remote_callback;
//
//             unsafe {
//                 remote_callback(host_c_str.as_ptr(), &mut start as *mut u32, &mut end as *mut u32);
//             }
//
//             (start as usize, end as usize)
//         }
//     }
//
//     adblock::url_parser::set_domain_resolver(Box::new(RemoteResolverImpl { remote_callback: resolver })).is_ok()
// }


/// Create a new `Engine`.
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineCreate(
    env: JNIEnv,
    _: JObject,
    rules: JString,
) -> jlong {
    check_init();
    let loadedRules = unwrapString(&env, rules);

    let mut filter_set = adblock::lists::FilterSet::new(false);
    debug!("Created filter_set with {:?}", &loadedRules);
    filter_set.add_filter_list(&loadedRules, adblock::lists::FilterFormat::Standard);
    let engine = Engine::from_filter_set(filter_set, true);
    Box::into_raw(Box::new(engine)) as jlong
}

/// Create a new `Engine`.
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineCreateDefault(
    env: JNIEnv,
    _: JObject,
) -> jlong {
    check_init();
    let engine = Engine::default();
    Box::into_raw(Box::new(engine)) as jlong
}

/// Checks if a `url` matches for the specified `Engine` within the context.
///
/// This API is designed for multi-engine use.
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_simpleMatch(
    env: JNIEnv,
    _: JObject,
    engine: jlong,
    url: JString,
    host: JString,
    resource_type: JString,
) -> jbyte
{
    check_init();
    let url = unwrapString(&env, url);
    let host = unwrapString(&env, host);
    let resource_type = unwrapString(&env, resource_type);
    let engine = unwrapEngine(&env, engine);

    let blocker_result = engine.check_network_urls(
        &url,
        &host,
        &resource_type,
    );
    debug!("New result for {:?} with {:?}", url, blocker_result);
    let mut result: i8 = 0;
    if blocker_result.matched { result |= IS_MATCHED_MASK; }
    if !blocker_result.exception.is_none() { result |= IS_EXCEPTION_MASK; }
    if blocker_result.important { result |= IS_IMPORTANT_MASK; };
    result
}



/// Checks if a `url` matches for the specified `Engine` within the context.
///
/// This API is designed for multi-engine use
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_match(
    env: JNIEnv,
    _: JObject,
    engine: jlong,
    url: JString,
    host: JString,
    tab_host: JString,
    third_party: bool,
    resource_type: JString,
    previous_result: jbyte,
) -> jbyte
{
    check_init();
    let url = unwrapString(&env, url);
    let host = unwrapString(&env, host);
    let tab_host = unwrapString(&env, tab_host);
    let resource_type = unwrapString(&env, resource_type);
    let engine = unwrapEngine(&env, engine);

    let was_matched = previous_result & IS_MATCHED_MASK != 0;
    let was_exception = previous_result & IS_EXCEPTION_MASK != 0;

    let blocker_result = engine.check_network_urls_with_hostnames_subset(
        &url,
        &host,
        &tab_host,
        &resource_type,
        Some(third_party),
        // Checking normal rules is skipped if a normal rule or exception rule was found previously
        was_matched || was_exception,
        // Always check exceptions unless one was found previously
        !was_exception,
    );
    debug!("New result for {:?} with {:?}", url, blocker_result);
    let mut result: i8 = 0;
    if blocker_result.matched { result |= IS_MATCHED_MASK; }
    if !blocker_result.exception.is_none() { result |= IS_EXCEPTION_MASK; }
    if blocker_result.important { result |= IS_IMPORTANT_MASK; };
    result
}


/// Adds a tag to the engine for consideration
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineEnableTag(
    env: JNIEnv,
    _: JObject,
    engine: jlong,
    tag: JString,
) {
    check_init();
    let tag = unwrapString(&env, tag);
    let engine = unwrapEngine(&env, engine);
    engine.enable_tags(&[&tag]);
}

/// Checks if a tag exists in the engine
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineTagExists(
    env: JNIEnv,
    _: JObject,
    engine: jlong,
    tag: JString,
) -> bool {
    check_init();
    let tag = unwrapString(&env, tag);
    let engine = unwrapEngine(&env, engine);
    let res = engine.tag_exists(&tag);
    debug!("Has tag {:?} {}", tag, res);
    res
}


/// Removes a tag to the engine for consideration
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineDisableTag(
    env: JNIEnv,
    _: JObject,
    engine: jlong,
    tag: JString,
) {
    check_init();
    let tag = unwrapString(&env, tag);
    let engine = unwrapEngine(&env, engine);
    engine.disable_tags(&[&tag]);
}

/// Adds a resource to the engine by name
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineAddResources(
    env: JNIEnv,
    _: JObject,
    engine: jlong,
    key: JString,
    contentType: JString,
    data: JString,
) -> bool {
    check_init();
    let key = unwrapString(&env, key);
    let contentType = unwrapString(&env, contentType);
    let data = unwrapString(&env, data);
    let engine = unwrapEngine(&env, engine);

    let resource = Resource {
        name: key.to_string(),
        aliases: vec![],
        kind: ResourceType::Mime(MimeType::from(std::borrow::Cow::from(contentType))),
        content: data.to_string(),
    };
    engine.add_resource(resource).is_ok()
}

/// Adds a list of `Resource`s from JSON format
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineAddResourceFromJson(
    env: JNIEnv,
    _: JObject,
    engine: jlong,
    resourcesJson: JString,
) {
    check_init();
    let resourcesJson = unwrapString(&env, resourcesJson);
    let engine = unwrapEngine(&env, engine);


    let resources: Vec<Resource> = serde_json::from_str(&resourcesJson).unwrap_or_else(|e| {
        error!("Failed to parse JSON adblock resources: {}", e);
        vec![]
    });

    engine.use_resources(&resources);
}


/// Deserializes a previously serialized data file list.
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineDeserialize(
    env: JNIEnv,
    _: JObject,
    engine: jlong,
    data: jbyteArray,
) -> bool {
    check_init();
    let data = env.convert_byte_array(data).unwrap();
    let engine = unwrapEngine(&env, engine);
    let ok = engine.deserialize(&data).is_ok();
    if !ok {
        eprintln!("Error deserializing adblock engine");
    }
    ok
}

/// Deserializes a previously serialized data file list from a local file path
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineDeserializeFromFile(
    env: JNIEnv,
    _: JObject,
    engine: jlong,
    filePath: JString,
) -> bool {
    check_init();
    let filePath = unwrapString(&env, filePath);
    let engine = unwrapEngine(&env, engine);
    debug!("Will try to deserialize engine from {:?}", filePath);
    let mut file = match File::open(&filePath) {
        Err(why) => {
            throwAndPanic!(&env, format!("couldn't open {:?}: {:?}", filePath, why))
        }
        Ok(file) => file,
    };
    let mut serialized = Vec::<u8>::new();
    file.read_to_end(&mut serialized)
        .expect("Reading from serialization file failed");
    let ok = engine.deserialize(&serialized).is_ok();
    if !ok {
        error!("Error deserializing adblock engine");
    }
    ok
}

/// Destroy a `Engine` once you are done with it.
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineDestroy(
    _env: JNIEnv,
    _: JObject,
    engine: jlong) {
    let enginePointer = engine as *mut Engine;

    if !enginePointer.is_null() {
        debug!("Will dispose of engine {:?}", enginePointer);
        drop(Box::from_raw(enginePointer));
    }
}
