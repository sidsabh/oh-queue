use crate::utils::*;
use actix_files as fs;

use crate::queue::*;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};

const ADDRESS: &str = "0.0.0.0";

/// This function starts the server and defines the routes for the web application.
pub async fn start_server(queue_ref: Arc<Mutex<Queue>>) -> std::io::Result<()> {
    let mut port = 3000;

    let queue_data = web::Data::new(queue_ref.clone());
    loop {
        let queue_data = queue_data.clone();
        let server = HttpServer::new(move || {
            App::new()
                .app_data(queue_data.clone())
                .route(
                    "/",
                    web::get().to(|| async {
                        let html = std::fs::read_to_string("src/public/index.html")
                            .expect("error reading index.html");
                        HttpResponse::Ok().content_type("text/html").body(html)
                    }),
                )
                .route(
                    "/waiting",
                    web::get().to(|| async {
                        let html = std::fs::read_to_string("src/public/waiting.html")
                            .expect("error reading waiting.html");
                        HttpResponse::Ok().content_type("text/html").body(html)
                    }),
                )
                .route(
                    "/done",
                    web::get().to(|| async {
                        let html = std::fs::read_to_string("src/public/done.html")
                            .expect("error reading done.html");
                        HttpResponse::Ok().content_type("text/html").body(html)
                    }),
                )
                .service(fs::Files::new("/static", "src/public").show_files_listing())
                .route("/api/join", web::post().to(join_queue))
                .route("/api/leave", web::post().to(leave_queue))
                .route("/api/position", web::get().to(get_position))
        });

        match server
            .bind(format!("{}:{}", ADDRESS, port))
            .map(|s| s.run())
        {
            Ok(server) => {
                println!("Serving on {}:{}", ADDRESS, port);
                log_server_details(port).await?; // Log server details to a file
                server.await?;
                return Ok(());
            }
            Err(_) => port += 1, // Increment the port if binding fails
        }

        
    }
}

use std::fs::{create_dir_all, metadata, set_permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

async fn log_server_details(port: u16) -> std::io::Result<()> {
    // Get the hostname
    let output = Command::new("hostname").output()?;
    let hostname = String::from_utf8_lossy(&output.stdout);

    // Get the home directory
    let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
    let dir_path = Path::new(&home_dir).join("Public");
    let file_path = dir_path.join("join-office-hours.txt");

    // Create the directory if it does not exist
    if !dir_path.exists() {
        create_dir_all(&dir_path)?;
        // Set directory permissions if needed
        let mut dir_permissions = metadata(&dir_path)?.permissions();
        dir_permissions.set_mode(0o755); // rwxr-xr-x
        set_permissions(&dir_path, dir_permissions)?;
    }

    // Open the file with append and create options
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&file_path)?;

    // Set file permissions if needed
    let mut file_permissions = file.metadata()?.permissions();
    file_permissions.set_mode(0o644); // rw-r--r--
    set_permissions(&file_path, file_permissions)?;

    // Log entry
    let log_entry = format!(
        "Connect via:\n\
        \tssh -N -L {}:{}:3000 <your-username>@<your-machine>\n\
        Visit http://localhost:3000 to join the office hours queue.\n",
        port, hostname.trim()
    );
    

    file.set_len(0)?;
    file.write_all(log_entry.as_bytes())?;

    Ok(())
}

async fn join_queue(
    data: web::Data<Arc<Mutex<Queue>>>,
    form: web::Form<StudentInfo>,
) -> HttpResponse {
    if form.name.is_empty()
        || form.csid.is_empty()
        || form.purpose.is_empty()
        || form.details.is_empty()
        || form.steps.is_empty()
    {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Please complete the requirements to join the office hour queue.");
    }

    let student_info = StudentInfo::new(
        form.name.clone(),
        form.csid.clone(),
        form.purpose.clone(),
        form.details.clone(),
        form.steps.clone(),
    );

    let student_request = StudentRequest::new(student_info);

    match handle_join(data, student_request.clone()) {
        Ok(_) => {
            // send to /waiting?id
            HttpResponse::Found()
                .append_header(("Location", format!("/waiting?id={}", student_request.id)))
                .finish()
        }
        Err(_) => HttpResponse::InternalServerError()
            .content_type("text/html")
            .body("An error occurred while processing your request."),
    }
}

/// This function handles the student request by appending to the queue.
fn handle_join(data: web::Data<Arc<Mutex<Queue>>>, request: StudentRequest) -> Result<(), ()> {
    let mut queue = data.lock().unwrap();
    queue.add(request);
    Ok(())
}

async fn leave_queue(
    data: web::Data<Arc<Mutex<Queue>>>,
    query: web::Query<IdQuery>,
) -> HttpResponse {
    match handle_leave(data, query.id.clone()) {
        Ok(_) => HttpResponse::Found()
            .append_header(("Location", "/done"))
            .finish(),
        Err(_) => HttpResponse::InternalServerError()
            .content_type("text/html")
            .body("An error occurred while processing your request."),
    }
}

fn handle_leave(data: web::Data<Arc<Mutex<Queue>>>, id: String) -> Result<(), ()> {
    let mut queue = data.lock().unwrap();
    queue.remove(id)
}

async fn get_position(
    data: web::Data<Arc<Mutex<Queue>>>,
    query: web::Query<IdQuery>,
) -> HttpResponse {
    println!("Position requested for ID: {}", query.id);
    match handle_position(data, query.id.clone()) {
        Ok(position) => HttpResponse::Ok().content_type("text/html").body(position),
        Err(_) => HttpResponse::InternalServerError()
            .content_type("text/html")
            .body("An error occurred while processing your request."),
    }
}

fn handle_position(data: web::Data<Arc<Mutex<Queue>>>, id: String) -> Result<String, ()> {
    let queue = data.lock().unwrap();
    match queue.position(id) {
        Ok(position) => Ok(position.to_string()),
        Err(_) => Err(()),
    }
}
