use rusqlite::Connection;
use std::path::Path;
use std::io::{self, Write};
use ring::digest;
use data_encoding::HEXLOWER;

fn hash_pin(pin: &str) -> String {
    let digest = digest::digest(&digest::SHA256, pin.as_bytes());
    HEXLOWER.encode(digest.as_ref())
}

#[derive(Debug)]
struct StaffMember {
    staff_id: String,
    first_name: String,
    last_name: String,
    hourly_wage: f64,
    is_admin: bool,
}

fn authenticate_with_pin(conn: &Connection, pin: &str) -> rusqlite::Result<Option<StaffMember>> {
    let hashed_pin = hash_pin(pin);
    
    let result = conn.query_row(
        "SELECT staff_id, first_name, last_name, hourly_wage, is_admin 
         FROM staff 
         WHERE pin = ?",
        [&hashed_pin],
        |row| {
            Ok(StaffMember {
                staff_id: row.get(0)?,
                first_name: row.get(1)?,
                last_name: row.get(2)?,
                hourly_wage: row.get(3)?,
                is_admin: row.get(4)?,
            })
        }
    );

    match result {
        Ok(data) => Ok(Some(data)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = Path::new("Database/pos.db");
    if !db_path.exists() {
        println!("Database file not found at: {}", db_path.display());
        return Ok(());
    }

    let conn = Connection::open(db_path)?;
    
    print!("Enter PIN (4 digits): ");
    io::stdout().flush()?;
    let mut pin = String::new();
    io::stdin().read_line(&mut pin)?;
    let pin = pin.trim();

    match authenticate_with_pin(&conn, pin)? {
        Some(staff) => {
            println!("\n=== Authentication Status ===");
            println!("✅ Successfully Authenticated!");
            println!("🔑 Admin Access: {}\n", if staff.is_admin { "YES ✨" } else { "NO" });
            
            println!("=== Staff Profile ===");
            println!("Full Name: {} {}", staff.first_name, staff.last_name);
            println!("Staff ID: {}", staff.staff_id);
            println!("Role: {}", if staff.is_admin { "Administrator" } else { "Staff Member" });
            println!("Hourly Wage: ${:.2}", staff.hourly_wage);
            
            if staff.is_admin {
                println!("\n=== Admin Privileges ===");
                println!("✓ Manage Staff");
                println!("✓ Manage Catalog");
                println!("✓ View Reports");
                println!("✓ Configure Settings");
                println!("✓ Manage Discounts");
            }
        }
        None => {
            println!("\n❌ Authentication Failed: Invalid PIN");
            println!("Please try again or contact an administrator for help.");
        }
    }

    Ok(())
}
