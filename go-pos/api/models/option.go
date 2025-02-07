package models

import (
	"database/sql"
	"errors"
	"pos-server/api/database"
	"pos-server/api/utils"
)

type Option struct {
	OptionID   string  `json:"option_id"`
	ModifierID string  `json:"modifier_id"`
	Name       string  `json:"name"`
	Price      float64 `json:"price"`
	Available  bool    `json:"available"`
	SortOrder  int     `json:"sort_order"`
}

func (o *Option) Create() error {
	if o.OptionID == "" {
		id, err := utils.GenerateOptionID(database.DB)
		if err != nil {
			return err
		}
		o.OptionID = id
	}

	query := `
		INSERT INTO options (option_id, modifier_id, name, price, available, sort_order)
		VALUES (?, ?, ?, ?, ?, ?)
	`

	_, err := database.DB.Exec(query,
		o.OptionID,
		o.ModifierID,
		o.Name,
		o.Price,
		o.Available,
		o.SortOrder,
	)

	return err
}

func GetOptionsByModifier(modifierID string) ([]Option, error) {
	rows, err := database.DB.Query(`
		SELECT option_id, modifier_id, name, price, available, sort_order 
		FROM options 
		WHERE modifier_id = ?
		ORDER BY sort_order
	`, modifierID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var options []Option
	for rows.Next() {
		var o Option
		err := rows.Scan(&o.OptionID, &o.ModifierID, &o.Name, &o.Price, &o.Available, &o.SortOrder)
		if err != nil {
			return nil, err
		}
		options = append(options, o)
	}
	return options, nil
}

func GetOptionByID(id string) (*Option, error) {
	var option Option
	err := database.DB.QueryRow(`
		SELECT option_id, modifier_id, name, price, available, sort_order 
		FROM options 
		WHERE option_id = ?
	`, id).Scan(&option.OptionID, &option.ModifierID, &option.Name, &option.Price, &option.Available, &option.SortOrder)
	if err == sql.ErrNoRows {
		return nil, errors.New("option not found")
	}
	if err != nil {
		return nil, err
	}
	return &option, nil
}

func (o *Option) Update() error {
	query := `
		UPDATE options 
		SET modifier_id = ?, name = ?, price = ?, available = ?, sort_order = ?
		WHERE option_id = ?
	`

	result, err := database.DB.Exec(query,
		o.ModifierID,
		o.Name,
		o.Price,
		o.Available,
		o.SortOrder,
		o.OptionID,
	)
	if err != nil {
		return err
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return err
	}
	if rows == 0 {
		return errors.New("option not found")
	}

	return nil
}

func DeleteOption(id string) error {
	result, err := database.DB.Exec("DELETE FROM options WHERE option_id = ?", id)
	if err != nil {
		return err
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return err
	}
	if rows == 0 {
		return errors.New("option not found")
	}

	return nil
}
