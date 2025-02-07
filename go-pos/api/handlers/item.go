package handlers

import (
	"pos-server/api/models"
	"github.com/gofiber/fiber/v2"
)

func HandleCreateItem(c *fiber.Ctx) error {
	var item models.Item
	if err := c.BodyParser(&item); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid request body",
		})
	}

	if err := item.Create(); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to create item",
		})
	}

	return c.Status(fiber.StatusCreated).JSON(item)
}

func HandleGetAllItems(c *fiber.Ctx) error {
	items, err := models.GetAllItems()
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to get items",
		})
	}

	return c.JSON(items)
}

func HandleGetItemsByCategory(c *fiber.Ctx) error {
	categoryID := c.Params("categoryId")
	items, err := models.GetItemsByCategory(categoryID)
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to get items",
		})
	}

	return c.JSON(items)
}

func HandleGetItem(c *fiber.Ctx) error {
	id := c.Params("id")
	item, err := models.GetItemByID(id)
	if err != nil {
		return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
			"error": "item not found",
		})
	}

	return c.JSON(item)
}

func HandleUpdateItem(c *fiber.Ctx) error {
	id := c.Params("id")
	var item models.Item
	if err := c.BodyParser(&item); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid request body",
		})
	}

	item.ItemID = id
	if err := item.Update(); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to update item",
		})
	}

	return c.JSON(item)
}

func HandleDeleteItem(c *fiber.Ctx) error {
	id := c.Params("id")
	if err := models.DeleteItem(id); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to delete item",
		})
	}

	return c.SendStatus(fiber.StatusNoContent)
}
