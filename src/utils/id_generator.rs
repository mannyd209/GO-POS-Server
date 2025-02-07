use rand::Rng;
use rusqlite::Connection;

pub enum IdType {
    Staff,
    Item,
    Category,
    Modifier,
    Option,
    Discount,
}

impl IdType {
    fn prefix(&self) -> &'static str {
        match self {
            IdType::Staff => "staff",
            IdType::Item => "item",
            IdType::Category => "category",
            IdType::Modifier => "modifier",
            IdType::Option => "option",
            IdType::Discount => "discount",
        }
    }

    fn table_name(&self) -> &'static str {
        match self {
            IdType::Staff => "staff",
            IdType::Item => "items",
            IdType::Category => "categories",
            IdType::Modifier => "modifiers",
            IdType::Option => "options",
            IdType::Discount => "discounts",
        }
    }

    fn id_column(&self) -> &'static str {
        match self {
            IdType::Staff => "staff_id",
            IdType::Item => "item_id",
            IdType::Category => "category_id",
            IdType::Modifier => "modifier_id",
            IdType::Option => "option_id",
            IdType::Discount => "discount_id",
        }
    }
}

pub fn generate_id(id_type: IdType, conn: &Connection) -> String {
    let mut rng = rand::thread_rng();
    loop {
        let random_num: u32 = rng.gen_range(100000..999999);
        let id = format!("{}{}", id_type.prefix(), random_num);
        
        // Check if ID exists in the respective table
        let exists: bool = conn.query_row(
            &format!(
                "SELECT 1 FROM {} WHERE {} = ?1",
                id_type.table_name(),
                id_type.id_column()
            ),
            [&id],
            |_| Ok(true)
        ).unwrap_or(false);
        
        if !exists {
            return id;
        }
    }
}
