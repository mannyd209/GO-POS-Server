package models

import (
	"database/sql"
	"errors"
	"pos-server/api/database"
	"pos-server/api/utils"
)

type Modifier struct {
	ModifierID      string `json:"modifier_id"`
	ItemID          string `json:"item_id"`
	Name            string `json:"name"`
	SingleSelection bool   `json:"single_selection"`
	SortOrder       int    `json:"sort_order"`
}

func (m *Modifier) Create() error {
	if m.ModifierID == "" {
		id, err := utils.GenerateModifierID(database.DB)
		if err != nil {
			return err
		}
		m.ModifierID = id
	}

	query := `
		INSERT INTO modifiers (modifier_id, item_id, name, single_selection, sort_order)
		VALUES (?, ?, ?, ?, ?)
	`

	_, err := database.DB.Exec(query,
		m.ModifierID,
		m.ItemID,
		m.Name,
		m.SingleSelection,
		m.SortOrder,
	)

	return err
}

func GetModifiersByItem(itemID string) ([]Modifier, error) {
	rows, err := database.DB.Query(`
		SELECT modifier_id, item_id, name, single_selection, sort_order 
		FROM modifiers 
		WHERE item_id = ?
		ORDER BY sort_order
	`, itemID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var modifiers []Modifier
	for rows.Next() {
		var m Modifier
		err := rows.Scan(&m.ModifierID, &m.ItemID, &m.Name, &m.SingleSelection, &m.SortOrder)
		if err != nil {
			return nil, err
		}
		modifiers = append(modifiers, m)
	}
	return modifiers, nil
}

func GetModifierByID(id string) (*Modifier, error) {
	var modifier Modifier
	err := database.DB.QueryRow(`
		SELECT modifier_id, item_id, name, single_selection, sort_order 
		FROM modifiers 
		WHERE modifier_id = ?
	`, id).Scan(&modifier.ModifierID, &modifier.ItemID, &modifier.Name, &modifier.SingleSelection, &modifier.SortOrder)
	if err == sql.ErrNoRows {
		return nil, errors.New("modifier not found")
	}
	if err != nil {
		return nil, err
	}
	return &modifier, nil
}

func (m *Modifier) Update() error {
	query := `
		UPDATE modifiers 
		SET item_id = ?, name = ?, single_selection = ?, sort_order = ?
		WHERE modifier_id = ?
	`

	result, err := database.DB.Exec(query,
		m.ItemID,
		m.Name,
		m.SingleSelection,
		m.SortOrder,
		m.ModifierID,
	)
	if err != nil {
		return err
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return err
	}
	if rows == 0 {
		return errors.New("modifier not found")
	}

	return nil
}

func DeleteModifier(id string) error {
	result, err := database.DB.Exec("DELETE FROM modifiers WHERE modifier_id = ?", id)
	if err != nil {
		return err
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return err
	}
	if rows == 0 {
		return errors.New("modifier not found")
	}

	return nil
}
