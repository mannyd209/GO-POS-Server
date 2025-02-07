package models

import (
	"database/sql"
	"errors"
	"go-pos/api/database"
	"go-pos/api/utils"
)

type Discount struct {
	DiscountID    string  `json:"discount_id"`
	Name          string  `json:"name"`
	IsPercentage  bool    `json:"is_percentage"`
	Amount        float64 `json:"amount"`
	Available     bool    `json:"available"`
	SortOrder     int     `json:"sort_order"`
}

func (d *Discount) Create() error {
	if d.DiscountID == "" {
		id, err := utils.GenerateDiscountID(database.DB)
		if err != nil {
			return err
		}
		d.DiscountID = id
	}

	query := `
		INSERT INTO discounts (discount_id, name, is_percentage, amount, available, sort_order)
		VALUES (?, ?, ?, ?, ?, ?)
	`

	_, err := database.DB.Exec(query,
		d.DiscountID,
		d.Name,
		d.IsPercentage,
		d.Amount,
		d.Available,
		d.SortOrder,
	)

	return err
}

func GetAllDiscounts() ([]Discount, error) {
	rows, err := database.DB.Query(`
		SELECT discount_id, name, is_percentage, amount, available, sort_order 
		FROM discounts 
		ORDER BY sort_order
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var discounts []Discount
	for rows.Next() {
		var d Discount
		err := rows.Scan(&d.DiscountID, &d.Name, &d.IsPercentage, &d.Amount, &d.Available, &d.SortOrder)
		if err != nil {
			return nil, err
		}
		discounts = append(discounts, d)
	}
	return discounts, nil
}

func GetDiscountByID(id string) (*Discount, error) {
	var discount Discount
	err := database.DB.QueryRow(`
		SELECT discount_id, name, is_percentage, amount, available, sort_order 
		FROM discounts 
		WHERE discount_id = ?
	`, id).Scan(&discount.DiscountID, &discount.Name, &discount.IsPercentage, &discount.Amount, &discount.Available, &discount.SortOrder)
	if err == sql.ErrNoRows {
		return nil, errors.New("discount not found")
	}
	if err != nil {
		return nil, err
	}
	return &discount, nil
}

func (d *Discount) Update() error {
	query := `
		UPDATE discounts 
		SET name = ?, is_percentage = ?, amount = ?, available = ?, sort_order = ?
		WHERE discount_id = ?
	`

	result, err := database.DB.Exec(query,
		d.Name,
		d.IsPercentage,
		d.Amount,
		d.Available,
		d.SortOrder,
		d.DiscountID,
	)
	if err != nil {
		return err
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return err
	}
	if rows == 0 {
		return errors.New("discount not found")
	}

	return nil
}

func DeleteDiscount(id string) error {
	result, err := database.DB.Exec("DELETE FROM discounts WHERE discount_id = ?", id)
	if err != nil {
		return err
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return err
	}
	if rows == 0 {
		return errors.New("discount not found")
	}

	return nil
}
