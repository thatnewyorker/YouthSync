use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use chrono::{Datelike, NaiveDate}; // For date parsing and manipulation
use csv::Writer; // For CSV export
use serde::{Deserialize, Serialize}; // Serialization support
use sqlx::{FromRow, SqlitePool}; // Async SQLX ORM

// Define a struct for attendance record in database
#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Attendance {
    student_id: i32,
    date: String,   // Format: YYYY-MM-DD
    status: String, // "Present" or "Absent"
}

// Define response structure for weekly report
#[derive(Debug, Serialize)]
struct WeeklyReport {
    week: String,
    present_count: i32,
}

/// Root endpoint handler, return API info
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .body("YouthSync API: Use /attendance (POST), /report (GET), or /export (GET)")
}

/// Handler for adding attendance records
async fn add_attendance(
    data: web::Json<Attendance>,
    pool: web::Data<SqlitePool>,
) -> impl Responder {
    let result = sqlx::query("INSERT INTO attendance (student_id, date, status) VALUES (?, ?, ?)")
        .bind(data.student_id)
        .bind(&data.date)
        .bind(&data.status)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Attendance recorded"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

/// Handler to generate weekly attendance report
async fn get_report(pool: web::Data<SqlitePool>) -> impl Responder {
    let records = sqlx::query_as::<_, Attendance>("SELECT * FROM attendance")
        .fetch_all(pool.get_ref())
        .await;

    match records {
        Ok(records) => {
            let mut weekly_counts: Vec<WeeklyReport> = vec![];
            for record in records {
                // Parse date string to NaiveDate
                let date = match NaiveDate::parse_from_str(&record.date, "%Y-%m-%d") {
                    Ok(date) => date,
                    Err(_) => continue, // Skip malformed dates silently or log error
                };

                // let week_key = format!("{}-W{:03}", iso_week.year(), iso_week.week());

                // Generate date in the format "MM/DD/YYYY"
                let week_key = format!("{:02}/{:02}/{}", date.month(), date.day(), date.year());

                // Aggregate presence count
                let existing_entry = weekly_counts
                    .iter_mut()
                    .find(|entry| entry.week == week_key);

                match existing_entry {
                    Some(entry) => {
                        if record.status == "Present" {
                            entry.present_count += 1;
                        }
                    }
                    None => {
                        weekly_counts.push(WeeklyReport {
                            week: week_key,
                            present_count: if record.status == "Present" { 1 } else { 0 },
                        });
                    }
                }
            }
            HttpResponse::Ok().json(weekly_counts)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

/// Endpoint to export attendance as CSV
async fn export_csv(pool: web::Data<SqlitePool>) -> impl Responder {
    let records = sqlx::query_as::<_, Attendance>("SELECT * FROM attendance")
        .fetch_all(pool.get_ref())
        .await;

    match records {
        Ok(records) => {
            // Setup CSV writer
            let mut wtr = Writer::from_writer(vec![]);
            wtr.write_record(["Student ID", "Date", "Status"]).unwrap();

            for record in records {
                wtr.write_record(&[record.student_id.to_string(), record.date, record.status])
                    .unwrap();
            }
            HttpResponse::Ok()
                .content_type("text/csv")
                .body(wtr.into_inner().unwrap())
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Current directory: {:?}", std::env::current_dir());

    // Create and connect to SQLite database
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
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Database connection failed",
            ));
        }
    };

    // Run migrations if necessary
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("Failed to run migrations: {}", e);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Migration failed",
        ));
    }

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive()) // Allow all origins for simplicity
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::get().to(index))
            .route("/attendance", web::post().to(add_attendance))
            .route("/report", web::get().to(get_report))
            .route("/export", web::get().to(export_csv))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
