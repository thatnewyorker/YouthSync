use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use chrono::{Datelike, NaiveDate};
use csv::Writer;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Attendance {
    student_id: i32,
    date: String,   // Format: YYYY-MM-DD
    status: String, // "Present" or "Absent"
}

#[derive(Debug, Serialize)]
struct WeeklyReport {
    week: String,
    present_count: i32,
}

async fn index() -> impl Responder {
    HttpResponse::Ok()
        .body("YouthSync API: Use /attendance (POST), /report (GET), or /export (GET)")
}

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

async fn get_report(pool: web::Data<SqlitePool>) -> impl Responder {
    let records = sqlx::query_as::<_, Attendance>("SELECT * FROM attendance")
        .fetch_all(pool.get_ref())
        .await;

    match records {
        Ok(records) => {
            let mut weekly_counts: Vec<WeeklyReport> = vec![];
            for record in records {
                let date = match NaiveDate::parse_from_str(&record.date, "%Y-%m-%d") {
                    Ok(date) => date,
                    Err(e) => {
                        return HttpResponse::InternalServerError()
                            .body(format!("Date parse error: {}", e));
                    }
                };
                let week = format!("Week {}-{}", date.iso_week().year(), date.iso_week().week());
                let count = weekly_counts.iter_mut().find(|r| r.week == week);
                if let Some(report) = count {
                    if record.status == "Present" {
                        report.present_count += 1;
                    }
                } else {
                    weekly_counts.push(WeeklyReport {
                        week,
                        present_count: if record.status == "Present" { 1 } else { 0 },
                    });
                }
            }
            HttpResponse::Ok().json(weekly_counts)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

async fn export_csv(pool: web::Data<SqlitePool>) -> impl Responder {
    let records = sqlx::query_as::<_, Attendance>("SELECT * FROM attendance")
        .fetch_all(pool.get_ref())
        .await;

    match records {
        Ok(records) => {
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

    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("Failed to run migrations: {}", e);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Migration failed",
        ));
    }

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
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
