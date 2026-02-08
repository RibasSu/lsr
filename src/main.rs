use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use warp::Filter;
use notify::{Event, RecursiveMode, Watcher, Result as NotifyResult, RecommendedWatcher, Config};
use std::time::Duration;
use tokio::sync::broadcast;
use std::env;
use futures_util::{StreamExt, SinkExt};
use warp::ws::Message;
use std::fs;

const LIVE_RELOAD_SCRIPT: &str = r#"
<script>
(function() {
    const ws = new WebSocket('ws://localhost:3030/livereload');
    ws.onmessage = function(event) {
        if (event.data === 'reload') {
            location.reload();
        }
    };
    ws.onclose = function() {
        setTimeout(function() {
            location.reload();
        }, 1000);
    };
})();
</script>
</body>"#;

type Clients = Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).map(|s| PathBuf::from(s)).unwrap_or(std::env::current_dir().unwrap());

    let (tx, _rx) = broadcast::channel::<()>(100);
    let tx_watcher = tx.clone();

    let clients: Clients = Arc::new(Mutex::new(Vec::new()));
    let clients_notifier = clients.clone();

    // Spawn task to notify all clients when file changes occur
    tokio::spawn(async move {
        let mut rx = tx.subscribe();
        loop {
            if rx.recv().await.is_ok() {
                let mut clients_lock = clients_notifier.lock().unwrap();
                clients_lock.retain(|client| {
                    client.send(Message::text("reload")).is_ok()
                });
            }
        }
    });

    let watcher_path = path.clone();
    std::thread::spawn(move || {
        let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
            move |res: NotifyResult<Event>| {
                if let Ok(event) = res {
                    // Filter out non-modify events if desired
                    match event.kind {
                        notify::EventKind::Modify(_) | notify::EventKind::Create(_) | notify::EventKind::Remove(_) => {
                            println!("File change detected: {:?}", event.paths);
                            let _ = tx_watcher.send(());
                        }
                        _ => {}
                    }
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(500))
        ).unwrap();

        watcher.watch(&watcher_path, RecursiveMode::Recursive).unwrap();
        loop {
            std::thread::park();
        }
    });

    let clients_filter = warp::any().map(move || clients.clone());

    // WebSocket route
    let live_reload = warp::path("livereload")
        .and(warp::ws())
        .and(clients_filter.clone())
        .map(|ws: warp::ws::Ws, clients: Clients| {
            ws.on_upgrade(move |socket| async move {
                let (mut tx_ws, mut rx_ws) = socket.split();
                let (client_tx, mut client_rx) = tokio::sync::mpsc::unbounded_channel();

                // Add client to the list
                clients.lock().unwrap().push(client_tx);

                // Spawn task to forward messages from channel to WebSocket
                tokio::spawn(async move {
                    while let Some(msg) = client_rx.recv().await {
                        if tx_ws.send(msg).await.is_err() {
                            break;
                        }
                    }
                });

                // Keep connection alive by reading messages
                while let Some(_result) = rx_ws.next().await {
                    // Just keep the connection open
                }
            })
        });

    let serve_path = path.clone();

    // File serving route with HTML injection
    let files = warp::get()
        .and(warp::path::tail())
        .and(warp::any().map(move || serve_path.clone()))
        .and_then(serve_file);

    let routes = files.or(live_reload);

    println!("Serving on http://localhost:3030");
    println!("Directory: {}", path.display());

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn serve_file(tail: warp::path::Tail, base_path: PathBuf) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let mut file_path = base_path.clone();

    // Handle root path
    let path_str = tail.as_str();
    if path_str.is_empty() || path_str == "/" {
        file_path.push("index.html");
    } else {
        file_path.push(path_str);
    }

    // Security check: ensure the path is within base_path
    let canonical_base = base_path.canonicalize().map_err(|_| warp::reject::not_found())?;
    let canonical_file = match file_path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // If file doesn't exist, try adding index.html if it's a directory
            if file_path.is_dir() {
                file_path.push("index.html");
                file_path.canonicalize().map_err(|_| warp::reject::not_found())?
            } else {
                return Err(warp::reject::not_found());
            }
        }
    };

    if !canonical_file.starts_with(&canonical_base) {
        return Err(warp::reject::not_found());
    }

    // Read the file
    let content = fs::read(&canonical_file).map_err(|_| warp::reject::not_found())?;

    // Check if it's an HTML file
    let is_html = canonical_file.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("html") || s.eq_ignore_ascii_case("htm"))
        .unwrap_or(false);

    if is_html {
        // Inject live reload script
        let html_content = String::from_utf8_lossy(&content);
        let modified_html = if html_content.contains("</body>") {
            html_content.replace("</body>", LIVE_RELOAD_SCRIPT)
        } else {
            // If no </body> tag, append at the end
            format!("{}{}", html_content, LIVE_RELOAD_SCRIPT.replace("</body>", ""))
        };

        Ok(Box::new(warp::reply::html(modified_html)))
    } else {
        // Serve other files normally
        let mime_type = mime_guess::from_path(&canonical_file)
            .first_or_octet_stream()
            .to_string();

        Ok(Box::new(warp::reply::with_header(
            content,
            "Content-Type",
            mime_type,
        )))
    }
}

