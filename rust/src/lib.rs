#![allow(non_snake_case)]

use log::{error, debug};
#[cfg(target_os = "android")]
use android_logger;

use adblock::engine::Engine;
#[cfg(target_os = "android")]
use {android_logger::Config, log::Level};

use adblock::resources::{MimeType, Resource, ResourceType};
use jni::objects::{JObject, JString};
use jni::sys::{jbyte, jbyteArray, jlong};
use jni::JNIEnv;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Once;
#[cfg(not(target_os = "android"))]
use {env_logger::Builder, log::LevelFilter};
use std::panic::{catch_unwind, UnwindSafe};
use lazy_static::lazy_static;


const IS_MATCHED_MASK: i8 = 1;
const IS_IMPORTANT_MASK: i8 = 2;
const IS_EXCEPTION_MASK: i8 = 4;
const ERROR: jbyte = -1;
const NULL_POINTER: jlong = 0;
const TRUE: jbyte = 1;
const FALSE: jbyte = 0;


lazy_static! {
    static ref _INIT_LOG: Once = {
        let init = Once::new();
        #[cfg(target_os = "android")]
        init.call_once(|| {
         android_logger::init_once(Config::default().with_min_level(Level::Debug));
         });
        #[cfg(not(target_os = "android"))]
        init.call_once(|| {
        let mut builder = Builder::new();
        builder.filter_level(LevelFilter::Debug);
        builder.init();
        });

init
};
}

fn unwrapString(env: &JNIEnv, jString: JString) -> String {
    let loadedRules: String = match env.get_string(jString) {
        Err(why) => panic!("Could not convert JString to String: {}", why),
        Ok(str) => str.into(),
    };
    loadedRules
}

fn unwrapEngine<'a>(enginePointer: jlong) -> &'a mut Engine {
    let enginePointer = enginePointer as *mut Engine;
    let engine = if let Some(restored) = unsafe { enginePointer.as_mut() } {
        restored
    } else {
        panic!("Engine pointer is null!")
    };
    engine
}

fn catch_and_forward_exceptions_bool<F: FnOnce() -> bool + UnwindSafe>(env: &JNIEnv, f: F) -> jbyte {
    _catch_and_forward_exceptions(env, f, |v| { if v { TRUE } else { FALSE } }, ERROR)
}

fn catch_and_forward_exceptions_jlong<F: FnOnce() -> jlong + UnwindSafe>(env: &JNIEnv, f: F) -> jlong {
    _catch_and_forward_exceptions(env, f, |v| { v }, NULL_POINTER)
}

fn catch_and_forward_exceptions_jbyte<F: FnOnce() -> jbyte + UnwindSafe>(env: &JNIEnv, f: F) -> jbyte {
    _catch_and_forward_exceptions(env, f, |v| { v }, ERROR)
}

fn catch_and_forward_exceptions_void<F: FnOnce() -> R + UnwindSafe, R>(env: &JNIEnv, f: F) -> () {
    _catch_and_forward_exceptions(env, f, |_| { () }, ())
}

fn _catch_and_forward_exceptions<F: FnOnce() -> R + UnwindSafe, C: FnOnce(R) -> RES, R, RES>(env: &JNIEnv, f: F, convert: C, error_value: RES) -> RES {
    match catch_unwind(f) {
        Ok(v) => {
            convert(v)
        }
        Err(err) => {
            error!("Will throw panic: {:?}", err);
            env.throw(format!("{:?}", err)).unwrap_or_else(|err2| {
                error!("Could not throw error that initially was {:?} : {:?}", err, err2)
            });
            error_value
        }
    }
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
    catch_and_forward_exceptions_jlong(&env, || {
        let loadedRules = unwrapString(&env, rules);

        let engine = _engineCreate(&loadedRules);
        Box::into_raw(Box::new(engine)) as jlong
    })
}

fn _engineCreate(loadedRules: &String) -> Engine {
    let mut filter_set = adblock::lists::FilterSet::new(false);
    debug!("Created filter_set with {:?}", &loadedRules);
    filter_set.add_filter_list(&loadedRules, adblock::lists::FilterFormat::Standard);
    let engine = Engine::from_filter_set(filter_set, true);
    engine
}

/// Create a new `Engine`.
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineCreateDefault(
    env: JNIEnv,
    _: JObject,
) -> jlong {
    catch_and_forward_exceptions_jlong(&env, || {
        let engine = Engine::default();
        Box::into_raw(Box::new(engine)) as jlong
    })
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
) -> jbyte {
    catch_and_forward_exceptions_jbyte(&env, || {
        let url = unwrapString(&env, url);
        let host = unwrapString(&env, host);
        let resource_type = unwrapString(&env, resource_type);
        let engine = unwrapEngine(engine);

        _simpleMatch(&url, &host, &resource_type, engine)
    })
}


fn _simpleMatch(url: &String, host: &String, resource_type: &String, engine: &mut Engine) -> i8 {
    let blocker_result = engine.check_network_urls(&url, &host, &resource_type);
    debug!("New result for {:?} with {:?}", url, blocker_result);
    let mut result: i8 = 0;
    if blocker_result.matched {
        result |= IS_MATCHED_MASK;
    }
    if blocker_result.exception.is_some() {
        result |= IS_EXCEPTION_MASK;
    }
    if blocker_result.important {
        result |= IS_IMPORTANT_MASK;
    };
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
) -> jbyte {
    catch_and_forward_exceptions_jbyte(&env, || {
        let url = unwrapString(&env, url);
        let host = unwrapString(&env, host);
        let tab_host = unwrapString(&env, tab_host);
        let resource_type = unwrapString(&env, resource_type);
        let engine = unwrapEngine(engine);

        _match(third_party, previous_result, &url, &host, &tab_host, &resource_type, engine)
    })
}

fn _match(third_party: bool, previous_result: i8, url: &String, host: &String, tab_host: &String, resource_type: &String, engine: &mut Engine) -> i8 {
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
    if blocker_result.matched {
        result |= IS_MATCHED_MASK;
    }
    if blocker_result.exception.is_some() {
        result |= IS_EXCEPTION_MASK;
    }
    if blocker_result.important {
        result |= IS_IMPORTANT_MASK;
    };
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
    catch_and_forward_exceptions_void(&env, || {
        let tag = unwrapString(&env, tag);
        let engine = unwrapEngine(engine);
        engine.enable_tags(&[&tag]);
    });
}

/// Checks if a tag exists in the engine
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineTagExists(
    env: JNIEnv,
    _: JObject,
    engine: jlong,
    tag: JString,
) -> jbyte {
    catch_and_forward_exceptions_bool(&env, || {
        let tag = unwrapString(&env, tag);
        let engine = unwrapEngine(engine);
        let res = engine.tag_exists(&tag);
        debug!("Has tag {:?} {}", tag, res);
        res
    })
}

/// Removes a tag to the engine for consideration
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineDisableTag(
    env: JNIEnv,
    _: JObject,
    engine: jlong,
    tag: JString,
) {
    catch_and_forward_exceptions_void(&env, || {
        let tag = unwrapString(&env, tag);
        let engine = unwrapEngine(engine);
        engine.disable_tags(&[&tag]);
    });
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
) -> jbyte {
    catch_and_forward_exceptions_bool(&env, || {
        let key = unwrapString(&env, key);
        let contentType = unwrapString(&env, contentType);
        let data = unwrapString(&env, data);
        let engine = unwrapEngine(engine);

        _addResources(key, contentType, data, engine)
    })
}

fn _addResources(key: String, contentType: String, data: String, engine: &mut Engine) -> bool {
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
    catch_and_forward_exceptions_void(&env, || {
        let resourcesJson = unwrapString(&env, resourcesJson);
        let engine = unwrapEngine(engine);

        _addResourcesFromJson(&resourcesJson, engine);
    })
}

fn _addResourcesFromJson(resourcesJson: &String, engine: &mut Engine) {
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
) -> jbyte {
    catch_and_forward_exceptions_bool(&env, || {
        let data = env.convert_byte_array(data).unwrap();
        let engine = unwrapEngine(engine);
        _deserialize(&data, engine)
    })
}

fn _deserialize(data: &Vec<u8>, engine: &mut Engine) -> bool {
    match engine.deserialize(&data) {
        Ok(_) => true,
        Err(why) => {
            error!("Could not deserialize Engine because {:?}", why);
            false
        }
    }
}


/// Deserializes a previously serialized data file list from a local file path
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineDeserializeFromFile(
    env: JNIEnv,
    _: JObject,
    engine: jlong,
    filePath: JString,
) -> jbyte {
    catch_and_forward_exceptions_bool(&env, || {
        let filePath = unwrapString(&env, filePath);
        let engine = unwrapEngine(engine);
        _deserializeFromFile(&filePath, engine)
    })
}

fn _deserializeFromFile(filePath: &String, engine: &mut Engine) -> bool {
    debug!("Will try to deserialize engine from {:?}", filePath);
    let mut file = match File::open(&filePath) {
        Err(why) => panic!("couldn't open {:?}: {:?}", filePath, why),
        Ok(file) => file,
    };
    let mut serialized = Vec::<u8>::new();
    file.read_to_end(&mut serialized)
        .expect("Reading from serialization file failed");

    match engine.deserialize(&serialized) {
        Ok(_) => true,
        Err(why) => {
            error!("Could not deserialize Engine because {:?}", why);
            false
        }
    }
}

/// Destroy a `Engine` once you are done with it.
#[no_mangle]
pub unsafe extern "C" fn Java_com_xayn_adblockeraar_Adblock_engineDestroy(
    env: JNIEnv,
    _: JObject,
    engine: jlong,
) {
    catch_and_forward_exceptions_void(&env, || {
        let enginePointer = engine as *mut Engine;

        if !enginePointer.is_null() {
            debug!("Will dispose of engine {:?}", enginePointer);
            Box::from_raw(enginePointer);
        }
    });
}
