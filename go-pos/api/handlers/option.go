package handlers

import (
	"pos-server/api/models"
	"github.com/gofiber/fiber/v2"
)

func HandleCreateOption(c *fiber.Ctx) error {
	var option models.Option
	if err := c.BodyParser(&option); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid request body",
		})
	}

	if err := option.Create(); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to create option",
		})
	}

	return c.Status(fiber.StatusCreated).JSON(option)
}

func HandleGetOptionsByModifier(c *fiber.Ctx) error {
	modifierID := c.Params("modifierId")
	options, err := models.GetOptionsByModifier(modifierID)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to get options",
		})
	}

	return c.JSON(options)
}

func HandleGetOption(c *fiber.Ctx) error {
	id := c.Params("id")
	option, err := models.GetOptionByID(id)
	if err != nil {
		return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
			"error": "option not found",
		})
	}

	return c.JSON(option)
}

func HandleUpdateOption(c *fiber.Ctx) error {
	id := c.Params("id")
	var option models.Option
	if err := c.BodyParser(&option); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid request body",
		})
	}

	option.OptionID = id
	if err := option.Update(); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to update option",
		})
	}

	return c.JSON(option)
}

func HandleDeleteOption(c *fiber.Ctx) error {
	id := c.Params("id")
	if err := models.DeleteOption(id); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to delete option",
		})
	}

	return c.SendStatus(fiber.StatusNoContent)
}
