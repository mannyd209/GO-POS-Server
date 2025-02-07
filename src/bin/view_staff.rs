use rusqlite::Connection;
use std::path::Path;

fn print_staff_details(conn: &Connection) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(
        "SELECT 
            staff_id,
            first_name,
            last_name,
            hourly_wage,
            is_admin
         FROM staff
         ORDER BY is_admin DESC, last_name, first_name"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?, // staff_id
            row.get::<_, String>(1)?, // first_name
            row.get::<_, String>(2)?, // last_name
            row.get::<_, f64>(3)?,    // hourly_wage
            row.get::<_, bool>(4)?,   // is_admin
        ))
    })?;

    println!("\n=== Staff Members ===\n");
    println!("Admin Staff:");
    println!("------------");
    
    let mut has_printed_regular = false;

    for row in rows {
        let (staff_id, first_name, last_name, hourly_wage, is_admin) = row?;
        
        if !is_admin && !has_printed_regular {
            println!("\nRegular Staff:");
            println!("-------------");
            has_printed_regular = true;
        }
        
        println!(
            "{} {} ({})
    ID: {}
    Hourly Wage: ${:.2}
    Role: {}\n",
            first_name,
            last_name,
            if is_admin { "Admin" } else { "Staff" },
            staff_id,
            hourly_wage,
            if is_admin { "Administrator" } else { "Staff Member" }
        );
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = Path::new("Database/pos.db");
    if !db_path.exists() {
        println!("Database file not found at: {}", db_path.display());
        return Ok(());
    }

    let conn = Connection::open(db_path)?;
    print_staff_details(&conn)?;

    Ok(())
}
