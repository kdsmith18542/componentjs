//! Development server with Hot Module Replacement
//!
//! Provides a local development server with:
//! - Static file serving
//! - WebSocket-based HMR
//! - File watching and auto-rebuild

mod hmr;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use colored::Colorize;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info};

use crate::cli::DevServerOptions;
use crate::config::Config;

pub use hmr::HmrMessage;

/// Shared server state
struct ServerState {
    /// Project configuration
    config: Arc<Config>,
    
    /// HMR broadcast channel
    hmr_tx: broadcast::Sender<HmrMessage>,
    
    /// Whether HMR is enabled
    hmr_enabled: bool,
}

/// Development server
pub struct DevServer {
    /// Project configuration
    config: Arc<Config>,
    
    /// Server options
    options: DevServerOptions,
}

impl DevServer {
    /// Create a new development server
    pub fn new(config: Arc<Config>, options: DevServerOptions) -> Result<Self> {
        Ok(Self { config, options })
    }
    
    /// Start the development server
    pub async fn start(&self) -> Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.options.host, self.options.port)
            .parse()?;
        
        // Create HMR broadcast channel
        let (hmr_tx, _) = broadcast::channel::<HmrMessage>(100);
        
        // Create shared state
        let state = Arc::new(ServerState {
            config: self.config.clone(),
            hmr_tx: hmr_tx.clone(),
            hmr_enabled: self.options.hmr,
        });
        
        // Set up file watcher
        if self.options.hmr {
            self.setup_file_watcher(hmr_tx.clone())?;
        }
        
        // Build router
        let app = Router::new()
            .route("/", get(serve_index))
            .route("/*path", get(serve_file))
            .route("/__component_hmr", get(hmr::hmr_websocket))
            .layer(CorsLayer::permissive())
            .with_state(state);
        
        // Open browser if requested
        if self.options.open {
            let url = format!("http://{}", addr);
            if let Err(e) = webbrowser_open(&url) {
                debug!("Failed to open browser: {}", e);
            }
        }
        
        // Start server
        info!("Server listening on http://{}", addr);
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
    
    /// Set up file watching for HMR
    fn setup_file_watcher(&self, hmr_tx: broadcast::Sender<HmrMessage>) -> Result<()> {
        let root = self.config.root.clone();
        
        // Use a debouncer to avoid too many events
        let (tx, rx) = std::sync::mpsc::channel();
        
        let mut debouncer = new_debouncer(
            std::time::Duration::from_millis(100),
            tx,
        )?;
        
        // Watch the source directory
        debouncer.watcher().watch(&root, RecursiveMode::Recursive)?;
        
        // Spawn a thread to handle file change events
        // The debouncer is moved into the thread to keep it alive
        std::thread::spawn(move || {
            // Keep debouncer alive for the duration of the watcher
            let _debouncer = debouncer;
            
            loop {
                match rx.recv() {
                    Ok(Ok(events)) => {
                        for event in events {
                            handle_file_change(&event.path, &hmr_tx);
                        }
                    }
                    Ok(Err(e)) => {
                        error!("Watch error: {:?}", e);
                    }
                    Err(_) => {
                        // Channel closed, exit
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
}

/// Handle a file change event
fn handle_file_change(path: &PathBuf, hmr_tx: &broadcast::Sender<HmrMessage>) {
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    
    // Only handle relevant file types
    let is_relevant = matches!(
        extension,
        "js" | "ts" | "jsx" | "tsx" | "css" | "scss" | "html" | "vue" | "svelte"
    );
    
    if !is_relevant {
        return;
    }
    
    eprintln!(
        "  {} File changed: {}",
        "↻".yellow(),
        path.display().to_string().dimmed()
    );
    
    let message = if extension == "css" || extension == "scss" {
        HmrMessage::CssUpdate {
            path: path.display().to_string(),
        }
    } else {
        HmrMessage::FullReload {
            reason: format!("File changed: {}", path.display()),
        }
    };
    
    let _ = hmr_tx.send(message);
}

/// Serve the index.html file
async fn serve_index(State(state): State<Arc<ServerState>>) -> Response {
    let index_path = state.config.root.join("index.html");
    
    if index_path.exists() {
        match std::fs::read_to_string(&index_path) {
            Ok(mut content) => {
                // Inject HMR client if enabled
                if state.hmr_enabled {
                    content = inject_hmr_client(&content);
                }
                Html(content).into_response()
            }
            Err(e) => {
                error!("Failed to read index.html: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read index.html").into_response()
            }
        }
    } else {
        // Generate a default index.html
        let default_html = generate_default_index(&state.config, state.hmr_enabled);
        Html(default_html).into_response()
    }
}

/// Serve static files
async fn serve_file(
    State(state): State<Arc<ServerState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response {
    let file_path = state.config.root.join(&path);
    
    if !file_path.exists() {
        return (StatusCode::NOT_FOUND, format!("File not found: {}", path)).into_response();
    }
    
    // Determine content type
    let content_type = get_content_type(&file_path);
    
    match std::fs::read(&file_path) {
        Ok(content) => {
            let mut response = content.into_response();
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                content_type.parse().unwrap(),
            );
            response
        }
        Err(e) => {
            error!("Failed to read file {}: {}", path, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response()
        }
    }
}

/// Get content type for a file
fn get_content_type(path: &PathBuf) -> &'static str {
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    
    match extension {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "ts" | "tsx" | "jsx" => "application/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "eot" => "application/vnd.ms-fontobject",
        _ => "application/octet-stream",
    }
}

/// Inject HMR client script into HTML
fn inject_hmr_client(html: &str) -> String {
    let hmr_script = r#"
<script type="module">
// Component HMR Client
(function() {
  const ws = new WebSocket(`ws://${location.host}/__component_hmr`);
  
  ws.onmessage = function(event) {
    const message = JSON.parse(event.data);
    
    switch (message.type) {
      case 'full-reload':
        console.log('[Component] Full reload:', message.reason);
        location.reload();
        break;
        
      case 'css-update':
        console.log('[Component] CSS update:', message.path);
        // Find and reload CSS
        const links = document.querySelectorAll('link[rel="stylesheet"]');
        links.forEach(link => {
          const url = new URL(link.href);
          url.searchParams.set('t', Date.now());
          link.href = url.toString();
        });
        break;
        
      case 'connected':
        console.log('[Component] HMR connected');
        break;
    }
  };
  
  ws.onclose = function() {
    console.log('[Component] HMR disconnected, attempting to reconnect...');
    setTimeout(() => location.reload(), 1000);
  };
})();
</script>
"#;
    
    // Insert before </body> or at the end
    if let Some(pos) = html.rfind("</body>") {
        let mut result = html.to_string();
        result.insert_str(pos, hmr_script);
        result
    } else {
        format!("{}{}", html, hmr_script)
    }
}

/// Generate a default index.html
fn generate_default_index(config: &Config, hmr_enabled: bool) -> String {
    let entrypoint = config.entrypoints.values().next()
        .map(|p| p.as_str())
        .unwrap_or("src/main.js");
    
    let hmr_script = if hmr_enabled {
        inject_hmr_client("")
    } else {
        String::new()
    };
    
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{}</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/{}"></script>
    {}
  </body>
</html>
"#,
        config.project.name,
        entrypoint,
        hmr_script
    )
}

/// Open URL in browser (simple implementation)
fn webbrowser_open(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()?;
    }
    
    Ok(())
}
