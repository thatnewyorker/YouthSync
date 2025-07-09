// YouthSync: A simple attendance tracking API in Rust using Actix Web and SQLite.
// Provides endpoints to record attendance, generate daily attendance reports, and export data as CSV.

use actix_cors::Cors;                  // Enable Cross-Origin Resource Sharing (CORS) for HTTP requests
use actix_web::{App, HttpResponse, HttpServer, Responder, web}; // Actix Web framework components
use chrono::{Datelike, NaiveDate};     // Date handling utilities
use csv::Writer;                       // CSV writer for exporting records
use serde::{Deserialize, Serialize};   // Serialization / deserialization for JSON and CSV
use sqlx::{FromRow, SqlitePool};       // Async SQLite DB pool and mapping from query rows

// Attendance represents a single attendance record in the database and in API requests.
#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Attendance {
    student_id: i32,
    date: String,   // Date in "YYYY-MM-DD" format
    status: String, // "Present" or "Absent"
}

// DailyReport represents aggregated attendance counts for a specific date.
#[derive(Debug, Serialize)]
struct DailyReport {
    date: String,       // Date in "MM-DD-YYYY" format for client readability
    present_count: i32, // Number of students present
    absent_count: i32,  // Number of students absent
}

// Root handler: provides basic API usage info.
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .body("YouthSync API: Use /attendance (POST), /report (GET), or /export (GET)")
}

// POST /attendance
// Accepts JSON payload to insert a new attendance record into the database.
async fn add_attendance(
    data: web::Json<Attendance>,
    pool: web::Data<SqlitePool>,
) -> impl Responder {
    // Execute INSERT query with bound parameters from JSON request.
    let result = sqlx::query("INSERT INTO attendance (student_id, date, status) VALUES (?, ?, ?)")
        .bind(data.student_id)
        .bind(&data.date)
        .bind(&data.status)
        .execute(pool.get_ref())
        .await;

    // Return OK on success or InternalServerError with error message on failure.
    match result {
        Ok(_) => HttpResponse::Ok().body("Attendance recorded"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

// GET /report
// Retrieves all attendance records, aggregates by day, and returns JSON array of DailyReport.
async fn get_report(pool: web::Data<SqlitePool>) -> impl Responder {
    // Fetch all rows from 'attendance' table into Attendance structs.
    let records = sqlx::query_as::<_, Attendance>("SELECT * FROM attendance")
        .fetch_all(pool.get_ref())
        .await;

    match records {
        Ok(records) => {
            let mut daily_counts: Vec<DailyReport> = Vec::new();

            for record in records {
                // Parse the stored date string into NaiveDate for formatting.
                let date = match NaiveDate::parse_from_str(&record.date, "%Y-%m-%d") {
                    Ok(date) => date,
                    Err(e) => {
                        return HttpResponse::InternalServerError()
                            .body(format!("Date parse error: {}", e));
                    }
                };
                // Format date as "MM-DD-YYYY" for response.
                let formatted_date = format!("{:02}-{:02}-{}", date.month(), date.day(), date.year());

                // Look for an existing entry for this date.
                if let Some(report) = daily_counts.iter_mut().find(|r| r.date == formatted_date) {
                    // Increment appropriate counter based on status.
                    match record.status.as_str() {
                        "Present" => report.present_count += 1,
                        "Absent" => report.absent_count += 1,
                        _ => (), // Skip invalid status values
                    }
                } else {
                    // Create a new report entry if none exists for this date.
                    daily_counts.push(DailyReport {
                        date: formatted_date.clone(),
                        present_count: if record.status == "Present" { 1 } else { 0 },
                        absent_count: if record.status == "Absent" { 1 } else { 0 },
                    });
                }
            }
            // Return aggregated report as JSON.
            HttpResponse::Ok().json(daily_counts)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

// GET /export
// Exports all attendance records as a CSV file download.
async fn export_csv(pool: web::Data<SqlitePool>) -> impl Responder {
    // Query all attendance records.
    let records = sqlx::query_as::<_, Attendance>("SELECT * FROM attendance")
        .fetch_all(pool.get_ref())
        .await;

    match records {
        Ok(records) => {
            // Initialize CSV writer over an in-memory buffer.
            let mut wtr = Writer::from_writer(vec![]);
            // Write CSV header row.
            wtr.write_record(["Student ID", "Date", "Status"]).unwrap();

            // Write each record as a new CSV row.
            for record in records {
                wtr.write_record(&[record.student_id.to_string(), record.date, record.status])
                    .unwrap();
            }

            // Return response with CSV content and proper content type.
            HttpResponse::Ok()
                .content_type("text/csv")
                .body(wtr.into_inner().unwrap())
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

// Main entry point: sets up database connection, runs migrations, and starts the HTTP server.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Print current working directory for debugging purposes.
    println!("Current directory: {:?}", std::env::current_dir());

    // Initialize SQLite connection pool, creating the DB file if missing.
    let pool = match sqlx::sqlite::SqlitePoolOptions::new()
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("./youthsync.db")
                .create_if_missing(true),
        )
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            // Return an error to abort startup.
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Database connection failed",
            ));
        }
    };

    // Execute SQL migrations located in the ./migrations directory.
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("Failed to run migrations: {}", e);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Migration failed",
        ));
    }

    // Build and run the Actix HTTP server.
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())           // Allow all CORS requests for simplicity.
            .app_data(web::Data::new(pool.clone())) // Share DB pool with handlers.
            .route("/", web::get().to(index))       // Root health-check / info endpoint.
            .route("/attendance", web::post().to(add_attendance)) // POST new attendance.
            .route("/report", web::get().to(get_report))         // GET aggregated report.
            .route("/export", web::get().to(export_csv))         // GET CSV export.
    })
    .bind("127.0.0.1:8080")? // Bind to localhost on port 8080.
    .run()
    .await
}
