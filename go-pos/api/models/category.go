package models

import (
	"database/sql"
	"errors"
	"go-pos/api/database"
	"go-pos/api/utils"
)

type Category struct {
	CategoryID string `json:"category_id"`
	Name       string `json:"name"`
	SortOrder  int    `json:"sort_order"`
}

func (c *Category) Create() error {
	if c.CategoryID == "" {
		id, err := utils.GenerateCategoryID(database.DB)
		if err != nil {
			return err
		}
		c.CategoryID = id
	}

	query := `
		INSERT INTO categories (category_id, name, sort_order)
		VALUES (?, ?, ?)
	`

	_, err := database.DB.Exec(query,
		c.CategoryID,
		c.Name,
		c.SortOrder,
	)

	return err
}

func GetAllCategories() ([]Category, error) {
	rows, err := database.DB.Query("SELECT category_id, name, sort_order FROM categories ORDER BY sort_order")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var categories []Category
	for rows.Next() {
		var c Category
		err := rows.Scan(&c.CategoryID, &c.Name, &c.SortOrder)
		if err != nil {
			return nil, err
		}
		categories = append(categories, c)
	}
	return categories, nil
}

func GetCategoryByID(id string) (*Category, error) {
	var category Category
	err := database.DB.QueryRow("SELECT category_id, name, sort_order FROM categories WHERE category_id = ?", id).
		Scan(&category.CategoryID, &category.Name, &category.SortOrder)
	if err == sql.ErrNoRows {
		return nil, errors.New("category not found")
	}
	if err != nil {
		return nil, err
	}
	return &category, nil
}

func (c *Category) Update() error {
	query := `
		UPDATE categories 
		SET name = ?, sort_order = ?
		WHERE category_id = ?
	`

	result, err := database.DB.Exec(query,
		c.Name,
		c.SortOrder,
		c.CategoryID,
	)
	if err != nil {
		return err
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return err
	}
	if rows == 0 {
		return errors.New("category not found")
	}

	return nil
}

func DeleteCategory(id string) error {
	result, err := database.DB.Exec("DELETE FROM categories WHERE category_id = ?", id)
	if err != nil {
		return err
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return err
	}
	if rows == 0 {
		return errors.New("category not found")
	}

	return nil
}
