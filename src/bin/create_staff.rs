use rusqlite::{Connection, OptionalExtension};
use std::path::Path;
use std::io::{self, Write};
use ring::digest;
use data_encoding::HEXLOWER;
use rand::Rng;

#[derive(Debug)]
struct StaffMember {
    staff_id: String,
    first_name: String,
    last_name: String,
    hourly_wage: f64,
    is_admin: bool,
}

fn hash_pin(pin: &str) -> String {
    let digest = digest::digest(&digest::SHA256, pin.as_bytes());
    HEXLOWER.encode(digest.as_ref())
}

fn authenticate_admin(conn: &Connection, pin: &str) -> rusqlite::Result<bool> {
    let hashed_pin = hash_pin(pin);
    
    let result: Option<bool> = conn.query_row(
        "SELECT is_admin FROM staff WHERE pin = ?",
        [&hashed_pin],
        |row| row.get(0),
    ).optional()?;

    Ok(result.unwrap_or(false))
}

fn generate_staff_id(conn: &Connection) -> rusqlite::Result<String> {
    let mut rng = rand::thread_rng();
    loop {
        let id = format!("staff{:06}", rng.gen_range(0..1000000));
        let exists: bool = conn.query_row(
            "SELECT 1 FROM staff WHERE staff_id = ?",
            [&id],
            |_| Ok(true),
        ).optional()?.unwrap_or(false);

        if !exists {
            return Ok(id);
        }
    }
}

fn check_pin_available(conn: &Connection, pin: &str) -> rusqlite::Result<bool> {
    let hashed_pin = hash_pin(pin);
    let exists: bool = conn.query_row(
        "SELECT 1 FROM staff WHERE pin = ?",
        [&hashed_pin],
        |_| Ok(true),
    ).optional()?.unwrap_or(false);

    Ok(!exists)
}

fn create_staff_member(conn: &Connection, first_name: &str, last_name: &str, pin: &str, hourly_wage: f64, is_admin: bool) -> rusqlite::Result<String> {
    let staff_id = generate_staff_id(conn)?;
    let hashed_pin = hash_pin(pin);

    conn.execute(
        "INSERT INTO staff (staff_id, pin, first_name, last_name, hourly_wage, is_admin) 
         VALUES (?, ?, ?, ?, ?, ?)",
        (staff_id.as_str(), hashed_pin.as_str(), first_name, last_name, hourly_wage, is_admin),
    )?;

    Ok(staff_id)
}

fn read_input(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = Path::new("Database/pos.db");
    if !db_path.exists() {
        println!("Database file not found at: {}", db_path.display());
        return Ok(());
    }

    let conn = Connection::open(db_path)?;
    
    // First authenticate as admin
    let admin_pin = read_input("Enter your admin PIN: ")?;
    
    if !authenticate_admin(&conn, &admin_pin)? {
        println!("\n❌ Authentication Failed: Invalid PIN or not an admin");
        return Ok(());
    }

    println!("\n✅ Admin Authentication Successful!");
    println!("=== Create New Staff Member ===\n");

    // Gather new staff member information
    let first_name = read_input("Enter first name: ")?;
    let last_name = read_input("Enter last name: ")?;
    
    let mut pin = String::new();
    loop {
        pin = read_input("Enter 4-digit PIN for new staff: ")?;
        if pin.len() != 4 || pin.parse::<u16>().is_err() {
            println!("PIN must be exactly 4 digits!");
            continue;
        }
        if check_pin_available(&conn, &pin)? {
            break;
        }
        println!("PIN already in use. Please choose a different PIN.");
    }

    let hourly_wage: f64 = loop {
        let wage_str = read_input("Enter hourly wage (e.g., 15.50): ")?;
        match wage_str.parse() {
            Ok(wage) if wage > 0.0 => break wage,
            _ => println!("Please enter a valid positive number!"),
        }
    };

    let is_admin = loop {
        match read_input("Should this staff member be an admin? (y/n): ")?.to_lowercase().as_str() {
            "y" | "yes" => break true,
            "n" | "no" => break false,
            _ => println!("Please enter 'y' or 'n'"),
        }
    };

    // Create the new staff member
    match create_staff_member(&conn, &first_name, &last_name, &pin, hourly_wage, is_admin) {
        Ok(staff_id) => {
            println!("\n✨ Staff Member Created Successfully! ✨");
            println!("=== New Staff Details ===");
            println!("Name: {} {}", first_name, last_name);
            println!("Staff ID: {}", staff_id);
            println!("Hourly Wage: ${:.2}", hourly_wage);
            println!("Role: {}", if is_admin { "Administrator" } else { "Staff Member" });
            println!("PIN: {}", pin);
            println!("\nPlease make sure to securely share these credentials with the new staff member.");
        }
        Err(e) => {
            println!("\n❌ Error creating staff member: {}", e);
        }
    }

    Ok(())
}
