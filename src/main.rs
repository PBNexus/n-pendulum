// src/main.rs
// This is the entry point of the Rust application, equivalent to app.py in the reference. It sets up the Actix-web server,
// configures routes for serving static files (HTML/CSS/JS) and the /simulate POST endpoint.
// The server binds to localhost:8000 and runs asynchronously. We use actix-files to serve the static folder,
// mirroring how Flask serves templates and static assets. No debug mode is enabled, matching the reference's debug=False.
// Assumptions: A 'static' folder exists in the project root containing index.html, style.css, and script.js.
// No authentication or error pages are added, keeping it minimal like the reference.

use actix_web::{web, App, HttpServer};
use actix_files::Files;
mod math;
mod logic;
mod ui;

#[actix_web::main]  // This macro bootstraps the async main function using Tokio, similar to how Python's if __name__ == '__main__' runs the app.
async fn main() -> std::io::Result<()> {  // Returns a std::io::Result to handle binding/listening errors.
    println!("Server running at http://localhost:8000");
    HttpServer::new(|| {  // Creates a new server factory closure that configures the app per worker thread.
        App::new()  // Initializes a new Actix App instance.
            .service(  // Registers the /simulate route group.
                web::resource("/simulate")  // Defines the path /simulate.
                    .route(web::post().to(ui::simulate_handler))  // Handles POST requests by calling the handler in ui.rs.
            )
            .service(Files::new("/", "./static")  // Serves files from the './static' directory at the root path '/'.
                .index_file("index.html")  // Defaults to serving index.html for '/' requests, like Flask's route('/').
                .use_last_modified(true)  // Uses file last-modified for caching, improving performance like in web apps.
            )  
            
        
    })
    .bind(("127.0.0.1", 8000))?  // Binds the server to localhost port 8000; ? propagates IO errors. Matches reference's default port.
    .run()  // Starts the server and blocks until shutdown.
    .await  // Awaits the server's future, handling graceful shutdown.
}