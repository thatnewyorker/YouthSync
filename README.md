🧮 YouthSync: Youth Program Attendance & Engagement Dashboard

![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)
![HTML](https://img.shields.io/badge/HTML-gray?logo=html5)
![License](https://img.shields.io/badge/License-MIT-blue)
![Build](https://img.shields.io/badge/build-passing-brightgreen)

Streamline attendance tracking and engagement analysis for youth programs with YouthSync, a web-based dashboard designed for community organizations.

This tool helps program coordinators monitor daily attendance, generate compliance reports, and gain insights — ideal for youth initiatives seeking efficient management solutions.



🚀 Why This Exists

Managing attendance for youth programs can be time-consuming and error-prone. YouthSync provides a lightweight, open-source solution to automate tracking, visualize trends, and export data, making it valuable for organizations like the YMCA or similar groups.

This project was created to showcase my technical skills and problem-solving abilities for potential employers in the youth program sector.



📦 Features





📊 Daily Attendance Tracker



⏱️ Real-Time Updates (via form input)



💸 Compliance Reporting (CSV export)



⚙️ Support for SQLite database



🦀 Built in Rust for performance, React for UI



📐 Example Inputs







Parameter



Value





Student ID



1





Date



07-08-2025





Status



Present

➡️ Outputs:





Daily attendance chart (mm-dd-yyyy format)



Updated student records



Exported CSV report



🛠️ Installation

git clone https://github.com/thatnewyorker/YouthSync
cd YouthSync
cargo build --release



🧪 Usage

cargo run --release





Navigate to the frontend folder: cd frontend



Serve the frontend: python3 -m http.server 3000



Open http://localhost:3000 in your browser.



Use the form to add attendance and view the daily chart.



Note: Ensure Rust, SQLite, and Python 3 are installed as prerequisites.



🔮 Roadmap





Engagement metrics (e.g., activity participation rates)



Multi-user support



Custom date range filters



Offline mode with local storage



Enhanced UI customization



🧠 Use Cases





Attendance management for youth camps



Compliance reporting for community programs



Data-driven insights for program planning



Demonstration of technical skills for job applications



🙌 Author

Gerard Cruzado
Created to showcase skills for youth program opportunities
Built with 💻 Rust + 🌐 React

🔧 Example

cargo run --release

This will start the backend server. Then, in another terminal:

cd frontend
python3 -m http.server 3000

Access http://localhost:3000 to use the dashboard.
