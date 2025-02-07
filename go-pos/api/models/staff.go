package models

import (
	"database/sql"
	"errors"
	"pos-server/api/database"
	"pos-server/api/utils"
	"regexp"
)

var (
	ErrInvalidPIN     = errors.New("PIN must be exactly 4 digits")
	ErrPINExists      = errors.New("PIN already exists")
	ErrStaffNotFound  = errors.New("staff not found")
)

type Staff struct {
	StaffID    string  `json:"staff_id"`
	PIN        string  `json:"pin"`
	FirstName  string  `json:"first_name"`
	LastName   string  `json:"last_name"`
	HourlyWage float64 `json:"hourly_wage"`
	IsAdmin    bool    `json:"is_admin"`
}

func validatePIN(pin string) bool {
	matched, _ := regexp.MatchString("^[0-9]{4}$", pin)
	return matched
}

func isPINUnique(pin string, excludeStaffID string) (bool, error) {
	var count int
	var err error
	if excludeStaffID == "" {
		err = database.DB.QueryRow("SELECT COUNT(*) FROM staff WHERE pin = ?", pin).Scan(&count)
	} else {
		err = database.DB.QueryRow("SELECT COUNT(*) FROM staff WHERE pin = ? AND staff_id != ?", pin, excludeStaffID).Scan(&count)
	}
	if err != nil {
		return false, err
	}
	return count == 0, nil
}

func (s *Staff) Create() error {
	// Validate PIN
	if !validatePIN(s.PIN) {
		return ErrInvalidPIN
	}

	// Check for duplicate PIN
	isUnique, err := isPINUnique(s.PIN, "")
	if err != nil {
		return err
	}
	if !isUnique {
		return ErrPINExists
	}

	if s.StaffID == "" {
		// Generate ID using firstName
		id, err := utils.GenerateStaffID(database.DB, s.FirstName)
		if err != nil {
			return err
		}
		s.StaffID = id
	}

	query := `
		INSERT INTO staff (staff_id, pin, first_name, last_name, hourly_wage, is_admin)
		VALUES (?, ?, ?, ?, ?, ?)
	`

	_, err = database.DB.Exec(query,
		s.StaffID,
		s.PIN,
		s.FirstName,
		s.LastName,
		s.HourlyWage,
		s.IsAdmin,
	)

	return err
}

func GetAllStaff() ([]Staff, error) {
	rows, err := database.DB.Query("SELECT staff_id, pin, first_name, last_name, hourly_wage, is_admin FROM staff")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var staff []Staff
	for rows.Next() {
		var s Staff
		err := rows.Scan(&s.StaffID, &s.PIN, &s.FirstName, &s.LastName, &s.HourlyWage, &s.IsAdmin)
		if err != nil {
			return nil, err
		}
		staff = append(staff, s)
	}
	return staff, nil
}

func GetStaffByID(id string) (*Staff, error) {
	var staff Staff
	err := database.DB.QueryRow("SELECT staff_id, pin, first_name, last_name, hourly_wage, is_admin FROM staff WHERE staff_id = ?", id).
		Scan(&staff.StaffID, &staff.PIN, &staff.FirstName, &staff.LastName, &staff.HourlyWage, &staff.IsAdmin)
	if err == sql.ErrNoRows {
		return nil, ErrStaffNotFound
	}
	if err != nil {
		return nil, err
	}
	return &staff, nil
}

func AuthenticateStaffByPIN(pin string) (*Staff, error) {
	if !validatePIN(pin) {
		return nil, ErrInvalidPIN
	}

	var staff Staff
	err := database.DB.QueryRow(`
		SELECT staff_id, pin, first_name, last_name, hourly_wage, is_admin 
		FROM staff 
		WHERE pin = ?
	`, pin).Scan(&staff.StaffID, &staff.PIN, &staff.FirstName, &staff.LastName, &staff.HourlyWage, &staff.IsAdmin)
	
	if err == sql.ErrNoRows {
		return nil, errors.New("invalid PIN")
	}
	if err != nil {
		return nil, err
	}
	return &staff, nil
}

func (s *Staff) Update() error {
	// Validate PIN
	if !validatePIN(s.PIN) {
		return ErrInvalidPIN
	}

	// Check for duplicate PIN
	isUnique, err := isPINUnique(s.PIN, s.StaffID)
	if err != nil {
		return err
	}
	if !isUnique {
		return ErrPINExists
	}

	query := `
		UPDATE staff 
		SET pin = ?, first_name = ?, last_name = ?, hourly_wage = ?, is_admin = ?
		WHERE staff_id = ?
	`

	result, err := database.DB.Exec(query,
		s.PIN,
		s.FirstName,
		s.LastName,
		s.HourlyWage,
		s.IsAdmin,
		s.StaffID,
	)
	if err != nil {
		return err
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return err
	}
	if rows == 0 {
		return ErrStaffNotFound
	}

	return nil
}

func DeleteStaff(id string) error {
	result, err := database.DB.Exec("DELETE FROM staff WHERE staff_id = ?", id)
	if err != nil {
		return err
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return err
	}
	if rows == 0 {
		return ErrStaffNotFound
	}

	return nil
}
