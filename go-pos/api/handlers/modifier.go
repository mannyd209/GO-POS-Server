package handlers

import (
	"go-pos/api/models"
	"github.com/gofiber/fiber/v2"
)

func HandleCreateModifier(c *fiber.Ctx) error {
	var modifier models.Modifier
	if err := c.BodyParser(&modifier); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid request body",
		})
	}

	if err := modifier.Create(); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to create modifier",
		})
	}

	return c.Status(fiber.StatusCreated).JSON(modifier)
}

func HandleGetModifiersByItem(c *fiber.Ctx) error {
	itemID := c.Params("itemId")
	modifiers, err := models.GetModifiersByItem(itemID)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to get modifiers",
		})
	}

	return c.JSON(modifiers)
}

func HandleGetModifier(c *fiber.Ctx) error {
	id := c.Params("id")
	modifier, err := models.GetModifierByID(id)
	if err != nil {
		return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
			"error": "modifier not found",
		})
	}

	return c.JSON(modifier)
}

func HandleUpdateModifier(c *fiber.Ctx) error {
	id := c.Params("id")
	var modifier models.Modifier
	if err := c.BodyParser(&modifier); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid request body",
		})
	}

	modifier.ModifierID = id
	if err := modifier.Update(); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to update modifier",
		})
	}

	return c.JSON(modifier)
}

func HandleDeleteModifier(c *fiber.Ctx) error {
	id := c.Params("id")
	if err := models.DeleteModifier(id); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to delete modifier",
		})
	}

	return c.SendStatus(fiber.StatusNoContent)
}
