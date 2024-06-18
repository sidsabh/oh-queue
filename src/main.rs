mod util;

use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer};
use util::*;

const ADDRESS: &str = "0.0.0.0:3000";

// TODO: add a terminal ui to start the server so the TA can customize their office hours details etc and post to the HTTP

/// This function starts the server and defines the routes for the web application.
#[actix_web::main]
async fn main() {
    let server = HttpServer::new(|| {
        App::new()
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
            .route("/api/position", web::get().to(get_position)) // Changed to GET to match typical RESTful semantics
    });

    println!("Serving on {}", ADDRESS);
    server
        .bind(ADDRESS)
        .expect("error binding server to address")
        .run()
        .await
        .expect("error running server");
}

/// This function is called when a student submits a request to join the office hour queue.
/// It takes a form as input and returns an HTTP response.
async fn join_queue(form: web::Form<StudentInfo>) -> HttpResponse {
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

    match handle_join(student_request.clone()) {
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
fn handle_join(student_request: StudentRequest) -> Result<(), ()> {
    // to do:
    // figure out how to have a file-based queue
    // append to the queue
    // return Ok(()) if successful
    // return Err(()) if unsuccessful
    println!("Request created {:?}", student_request);
    Ok(())
}

async fn leave_queue(query: web::Query<IdQuery>) -> HttpResponse {
    match handle_leave(query.id.clone()) {
        Ok(_) => HttpResponse::Found().append_header(("Location", "/done")).finish(),
        Err(_) => HttpResponse::InternalServerError()
            .content_type("text/html")
            .body("An error occurred while processing your request."),
    }
}


fn handle_leave(id: String) -> Result<(), ()> {
    // to do:
    // figure out how to have a file-based queue
    // remove from the queue
    // return Ok(()) if successful
    // return Err(()) if unsuccessful
    println!("Request removed for ID: {}", id);
    Ok(())
}

async fn get_position(query: web::Query<IdQuery>) -> HttpResponse {
    println!("Position requested for ID: {}", query.id);
    match handle_position(query.id.clone()) {
        Ok(position) => HttpResponse::Ok().content_type("text/html").body(position),
        Err(_) => HttpResponse::InternalServerError()
            .content_type("text/html")
            .body("An error occurred while processing your request."),
    }
}

fn handle_position(id: String) -> Result<String, ()> {
    // to do:
    // figure out how to have a file-based queue
    // get the position of the student in the queue
    // return Ok(position) if successful
    // return Err(()) if unsuccessful
    println!("Position requested for ID: {}", id);
    Ok("1".to_string())
}
