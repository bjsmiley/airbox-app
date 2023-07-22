use fdcore::api::{self, CmdApi, QueryApi};
use fdcore::node::{CoreEvent, Node};
use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::io::BufReader;
use std::os::raw::c_char;
use std::sync::Arc;
use tokio::task::JoinHandle;

use once_cell::sync::{Lazy, OnceCell};
use tokio::{
    runtime::Runtime,
    sync::{mpsc, Mutex},
};

// extern "C" {
//     fn core_ready();
// }

// // lib.rs, simple FFI code
// #[no_mangle]
// pub extern "C" fn my_add(x: i32, y: i32) -> i32 {
//     x + y
// }

// fn send_core(msg: T, channel: mpsc::UnboundedSender<ReturnableMessage<T>>)

// #[no_mangle]
// pub extern "C" fn query(query: AppQuery, callback: extern "C" fn(CoreResponse)) {
//     RUNTIME.spawn(async move {
//         let node = {
//             let node = &mut *NODE.lock().await;
//             match node {
//                 Some(n) => n.clone(),
//                 None => init_node().await,
//             }
//         };
//         let resp = node.get_controller().query(msg).await.unwrap();
//         callback(resp);
//     });
// }

// lolol
// #[no_mangle]
// pub unsafe extern "C" fn set_peer_name(ptr: *mut Buffer) {
//     let buf = Box::from_raw(ptr);
//     let name = buf.into_string();
//     RUNTIME.spawn(async move {
//         let node = {
//             let node = &mut *NODE.lock().await;
//             match node {
//                 Some(n) => n.clone(),
//                 None => init_node().await,
//             }
//         };
//         let buf = {
//             let reply = node.get_cmd_api().send(msg).await.unwrap();
//         };
//         callback(Box::into_raw(Box::new(buf)));
//     });
// }

// fn send_cmd(msg: api::cmd::Request) -> api::cmd::Response {}

// #[no_mangle]
// pub unsafe extern "C" fn cmd(ptr: *mut Buffer, callback: extern "C" fn(*mut Buffer)) {
// let buf = Box::from_raw(ptr);
// let msg = serde_json::from_reader(buf.into_reader())
//     .map_err(|_| Error::Json)
//     .unwrap();
// RUNTIME.spawn(async move {
//     let node = {
//         let node = &mut *NODE.lock().await;
//         match node {
//             Some(n) => n.clone(),
//             None => init_node().await,
//         }
//     };
//     let buf = {
//         let reply = node.get_cmd_api().send(msg).await.unwrap();
//         Buffer::from_vec(serde_json::to_vec(&reply).unwrap())
//     };
//     callback(Box::into_raw(Box::new(buf)));
// });
// }

// #[no_mangle]
// pub extern "C" fn listen(on_event: extern "C" fn(*mut Buffer)) {
//     let (tx, mut rx) = mpsc::channel(100);
//     if EVENT_SENDER.set(tx).is_ok() {
//         RUNTIME.spawn(async move {
//             while let Some(e) = rx.recv().await {
//                 let buf = Buffer::from_vec(serde_json::to_vec(&e).unwrap());
//                 on_event(Box::into_raw(Box::new(buf)));
//             }
//         });
//     }
// }

#[no_mangle]
pub unsafe extern "C" fn init(
    data_dir: *const c_char,
    on_event: extern "C" fn(*const c_char),
    on_ready: extern "C" fn(),
) {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .init();

    if EVENT_LOOP.get().is_none() {
        let dir = CStr::from_ptr(data_dir).to_str().unwrap().to_string();
        RUNTIME.spawn(async move {
            let (mut node, mut rx) = Node::init(std::path::PathBuf::from(dir)).await.unwrap();

            RUNTIME.spawn(async move {
                while let Some(e) = rx.recv().await {
                    let json = serde_json::to_string(&e).unwrap();
                    let utf8json = CString::new(json.as_bytes()).unwrap();
                    on_event(utf8json.as_ptr());
                }
            });

            _ = API.set((node.get_query_api(), node.get_cmd_api()));
            _ = EVENT_LOOP.set(RUNTIME.spawn(async move { node.start().await }));
            on_ready();
        });
    }
}

#[no_mangle]
pub unsafe extern "C" fn query(msg: *const c_char, callback: extern "C" fn(*const c_char)) {
    let json = CStr::from_ptr(msg).to_str().unwrap();
    let req = serde_json::from_str(json).unwrap();
    let api = &API.get().unwrap().0;
    RUNTIME.spawn(async move {
        let result = api.send(req).await;
        let json = match result {
            Err(e) => serde_json::to_string(&QueryResponse::err(e.to_string())).unwrap(),
            Ok(res) => serde_json::to_string(&QueryResponse::body(res)).unwrap(),
        };
        // println!("from core: {}", json);
        let utf8json = CString::new(json.as_bytes()).unwrap();
        callback(utf8json.as_ptr());
    });
}

#[no_mangle]
pub unsafe extern "C" fn cmd(msg: *const c_char, callback: extern "C" fn(*const c_char)) {
    let json = CStr::from_ptr(msg).to_str().unwrap();
    let Ok(req) = serde_json::from_str(json) else {
        let res = serde_json::to_string(&CmdResponse::err(format!("failed to deser request: {}", json))).unwrap();
        let utf8json = CString::new(res.as_bytes()).unwrap();
        callback(utf8json.as_ptr());
        return;
    };

    let api = &API.get().unwrap().1;
    RUNTIME.spawn(async move {
        let result = api.send(req).await;
        let json = match result {
            Err(e) => serde_json::to_string(&CmdResponse::err(e.to_string())).unwrap(),
            Ok(res) => serde_json::to_string(&CmdResponse::body(res)).unwrap(),
        };
        // println!("from core: {}", json);
        let utf8json = CString::new(json.as_bytes()).unwrap();
        callback(utf8json.as_ptr());
    });
}

// #[allow(dead_code)]
pub(crate) static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

// #[allow(dead_code)]
// pub(crate) static NODE: Lazy<Mutex<Option<Arc<Node>>>> = Lazy::new(|| Mutex::new(None));

// #[allow(dead_code)]
// pub(crate) static CONTROLLER: Lazy<Mutex<Option<CoreController>>> = Lazy::new(|| Mutex::new(None));

// pub(crate) static EVENT_SENDER: OnceCell<mpsc::Sender<CoreEvent>> = OnceCell::new();
pub(crate) static EVENT_LOOP: OnceCell<JoinHandle<()>> = OnceCell::new();
pub(crate) static API: OnceCell<(QueryApi, CmdApi)> = OnceCell::new();

// async fn init_node() -> Arc<Node> {
//     let dir = dirs::config_dir().unwrap();
//     let (node, mut rx) = Node::init(dir).await.unwrap();
//     let node = Arc::new(node);
//     let n = &mut *NODE.lock().await;
//     n.replace(node.clone());
//     RUNTIME.spawn(async move {
//         while let Some(e) = rx.recv().await {
//             if let Some(tx) = EVENT_SENDER.get() {
//                 if tx.send(e).await.is_err() {
//                     break;
//                 }
//             }
//         }
//     });
//     node
// }

type QueryResponse = Response<api::query::Response, String>;
type CmdResponse = Response<api::cmd::Response, String>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Response<R, E> {
    pub err: Option<E>,
    pub res: Option<R>,
}

impl<R, E> Response<R, E> {
    pub fn err(e: E) -> Response<R, E> {
        Response {
            err: Some(e),
            res: None,
        }
    }
    pub fn body(r: R) -> Response<R, E> {
        Response {
            err: None,
            res: Some(r),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Json,
}

#[repr(C)]
pub struct Buffer {
    ptr: *mut u8,
    len: i32,
    cap: i32,
}

impl Buffer {
    pub fn len(&self) -> usize {
        self.len
            .try_into()
            .expect("buffer length negative or overflowed")
    }

    pub fn from_vec(bytes: Vec<u8>) -> Self {
        let len = i32::try_from(bytes.len()).expect("buffer length cannot fit into a i32.");
        let cap = i32::try_from(bytes.capacity()).expect("buffer capacity cannot fit into a i32.");

        // keep memory until call delete
        let mut v = std::mem::ManuallyDrop::new(bytes);

        Self {
            ptr: v.as_mut_ptr(),
            len,
            cap,
        }
    }

    pub fn into_string(self) -> String {
        unsafe { String::from_utf8_unchecked(self.into_vec()) }
    }

    /// Destorys the buffer data
    pub fn into_vec(self) -> Vec<u8> {
        if self.ptr.is_null() {
            vec![]
        } else {
            let cap: usize = self
                .cap
                .try_into()
                .expect("buffer capacity negative or overflowed");
            let len: usize = self
                .len
                .try_into()
                .expect("buffer length negative or overflowed");
            unsafe { Vec::from_raw_parts(self.ptr, len, cap) }
        }
    }

    pub fn into_reader(self) -> BufReader<VecDeque<u8>> {
        BufReader::new(self.into_vec().into())
    }

    pub fn destroy(self) {
        drop(self.into_vec());
    }
}
