// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{AppHandle, Emitter, EventTarget, Listener, Manager, ipc::Channel};
use serde::{Serialize, Deserialize};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}


// #[tauri::command]
// fn download(app: AppHandle, url: String) {
//   app.emit("download-started", &url).unwrap();
//   for progress in [1, 15, 50, 80, 100] {
//     app.emit("download-progress", progress).unwrap();
//   }
//   app.emit("download-finished", &url).unwrap();
// }

#[tauri::command]
fn login(app: AppHandle, user: String, password: String) {
  let authenticated = user == "tauri-apps" && password == "tauri";
  let result = if authenticated { "loggedIn" } else { "invalidCredentials" };
  app.emit_to("login", "login-result", result).unwrap();
}


#[tauri::command]
fn open_file(app: AppHandle, path: std::path::PathBuf) {
  app.emit_filter("open-file", path, |target| match target {
    EventTarget::WebviewWindow { label } => label == "main" || label == "file-viewer",
    _ => false,
  }).unwrap();
}


#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadStarted<'a> {
  url: &'a str,
  download_id: usize,
  content_length: usize,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadProgress {
  download_id: usize,
  chunk_length: usize,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadFinished {
  download_id: usize,
}

// #[tauri::command]
// fn download(app: AppHandle, url: String) {
//   let content_length = 1000;
//   let download_id = 1;

//   app.emit("download-started", DownloadStarted {
//     url: &url,
//     download_id,
//     content_length
//   }).unwrap();

//   for chunk_length in [15, 150, 35, 500, 300] {
//     app.emit("download-progress", DownloadProgress {
//       download_id,
//       chunk_length,
//     }).unwrap();
//   }

//   app.emit("download-finished", DownloadFinished { download_id }).unwrap();
// }





#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
enum DownloadEvent<'a> {
  #[serde(rename_all = "camelCase")]
  Started {
    url: &'a str,
    download_id: usize,
    content_length: usize,
  },
  #[serde(rename_all = "camelCase")]
  Progress {
    download_id: usize,
    chunk_length: usize,
  },
  #[serde(rename_all = "camelCase")]
  Finished {
    download_id: usize,
  },
}

#[tauri::command]
fn download(app: AppHandle, url: String, on_event: Channel<DownloadEvent>) {
  let content_length = 1000;
  let download_id = 1;

  on_event.send(DownloadEvent::Started {
    url: &url,
    download_id,
    content_length,
  }).unwrap();

  for chunk_length in [15, 150, 35, 500, 300] {
    on_event.send(DownloadEvent::Progress {
      download_id,
      chunk_length,
    }).unwrap();
  }

  on_event.send(DownloadEvent::Finished { download_id }).unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
    .setup(|app| {
        app.listen("download-started", |event| {
            if let Ok(payload) = serde_json::from_str::<DownloadStarted>(&event.payload()) {
            println!("downloading {}", payload.url);
            }
            
        });
        let webview = app.get_webview_window("main").unwrap();
        webview.listen("logged-in", |event| {
          let _session_token = event.payload();
          // save token..
        });
        
        let webview = app.get_webview_window("main").unwrap();
        webview.eval("console.log('hello from Rust')")?;
        Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet,download,login,open_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
