package models

import (
	"database/sql"
	"errors"
	"go-pos/api/database"
	"go-pos/api/utils"
)

type Item struct {
	ItemID       string  `json:"item_id"`
	CategoryID   string  `json:"category_id"`
	Name         string  `json:"name"`
	RegularPrice float64 `json:"regular_price"`
	EventPrice   float64 `json:"event_price"`
	SortOrder    int     `json:"sort_order"`
	Available    bool    `json:"available"`
}

func (i *Item) Create() error {
	if i.ItemID == "" {
		id, err := utils.GenerateItemID(database.DB)
		if err != nil {
			return err
		}
		i.ItemID = id
	}

	query := `
		INSERT INTO items (item_id, category_id, name, regular_price, event_price, sort_order, available)
		VALUES (?, ?, ?, ?, ?, ?, ?)
	`

	_, err := database.DB.Exec(query,
		i.ItemID,
		i.CategoryID,
		i.Name,
		i.RegularPrice,
		i.EventPrice,
		i.SortOrder,
		i.Available,
	)

	return err
}

func GetAllItems() ([]Item, error) {
	rows, err := database.DB.Query(`
		SELECT item_id, category_id, name, regular_price, event_price, sort_order, available 
		FROM items 
		ORDER BY sort_order
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var items []Item
	for rows.Next() {
		var i Item
		err := rows.Scan(
			&i.ItemID, &i.CategoryID, &i.Name, &i.RegularPrice, &i.EventPrice,
			&i.SortOrder, &i.Available,
		)
		if err != nil {
			return nil, err
		}
		items = append(items, i)
	}
	return items, nil
}

func GetItemsByCategory(categoryID string) ([]Item, error) {
	rows, err := database.DB.Query(`
		SELECT item_id, category_id, name, regular_price, event_price, sort_order, available 
		FROM items 
		WHERE category_id = ?
		ORDER BY sort_order
	`, categoryID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var items []Item
	for rows.Next() {
		var i Item
		err := rows.Scan(
			&i.ItemID, &i.CategoryID, &i.Name, &i.RegularPrice, &i.EventPrice,
			&i.SortOrder, &i.Available,
		)
		if err != nil {
			return nil, err
		}
		items = append(items, i)
	}
	return items, nil
}

func GetItemByID(id string) (*Item, error) {
	var item Item
	err := database.DB.QueryRow(`
		SELECT item_id, category_id, name, regular_price, event_price, sort_order, available 
		FROM items 
		WHERE item_id = ?
	`, id).Scan(
		&item.ItemID, &item.CategoryID, &item.Name, &item.RegularPrice, &item.EventPrice,
		&item.SortOrder, &item.Available,
	)
	if err == sql.ErrNoRows {
		return nil, errors.New("item not found")
	}
	if err != nil {
		return nil, err
	}
	return &item, nil
}

func (i *Item) Update() error {
	query := `
		UPDATE items 
		SET category_id = ?, name = ?, regular_price = ?, event_price = ?, 
			sort_order = ?, available = ?
		WHERE item_id = ?
	`

	result, err := database.DB.Exec(query,
		i.CategoryID,
		i.Name,
		i.RegularPrice,
		i.EventPrice,
		i.SortOrder,
		i.Available,
		i.ItemID,
	)
	if err != nil {
		return err
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return err
	}
	if rows == 0 {
		return errors.New("item not found")
	}

	return nil
}

func DeleteItem(id string) error {
	result, err := database.DB.Exec("DELETE FROM items WHERE item_id = ?", id)
	if err != nil {
		return err
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return err
	}
	if rows == 0 {
		return errors.New("item not found")
	}

	return nil
}
