package handlers

import (
	"pos-server/api/middleware"
	"pos-server/api/models"
	"github.com/gofiber/fiber/v2"
)

type loginRequest struct {
	PIN string `json:"pin"`
}

func HandleStaffLogin(c *fiber.Ctx) error {
	var req loginRequest
	if err := c.BodyParser(&req); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid request body",
		})
	}

	staff, err := models.AuthenticateStaffByPIN(req.PIN)
	if err != nil {
		return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{
			"error": "invalid PIN",
		})
	}

	// Set staff info in session
	store := middleware.GetStore()
	sess, err := store.Get(c)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "session error",
		})
	}

	sess.Set("staff_id", staff.StaffID)
	sess.Set("is_admin", staff.IsAdmin)
	if err := sess.Save(); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to save session",
		})
	}

	return c.JSON(staff)
}

func HandleCreateStaff(c *fiber.Ctx) error {
	var staff models.Staff
	if err := c.BodyParser(&staff); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid request body",
		})
	}

	if err := staff.Create(); err != nil {
		if err.Error() == "PIN already exists" {
			return c.Status(fiber.StatusConflict).JSON(fiber.Map{
				"error": err.Error(),
			})
		}
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to create staff",
		})
	}

	return c.Status(fiber.StatusCreated).JSON(staff)
}

func HandleGetAllStaff(c *fiber.Ctx) error {
	staff, err := models.GetAllStaff()
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to get staff",
		})
	}

	return c.JSON(staff)
}

func HandleGetStaff(c *fiber.Ctx) error {
	id := c.Params("id")
	staff, err := models.GetStaffByID(id)
	if err != nil {
		return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
			"error": "staff not found",
		})
	}

	return c.JSON(staff)
}

func HandleUpdateStaff(c *fiber.Ctx) error {
	id := c.Params("id")
	var staff models.Staff
	if err := c.BodyParser(&staff); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid request body",
		})
	}

	staff.StaffID = id
	if err := staff.Update(); err != nil {
		if err.Error() == "PIN already exists" {
			return c.Status(fiber.StatusConflict).JSON(fiber.Map{
				"error": err.Error(),
			})
		}
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to update staff",
		})
	}

	return c.JSON(staff)
}

func HandleDeleteStaff(c *fiber.Ctx) error {
	id := c.Params("id")
	if err := models.DeleteStaff(id); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to delete staff",
		})
	}

	return c.SendStatus(fiber.StatusNoContent)
}
