package handlers

import (
	"go-pos/api/models"
	"github.com/gofiber/fiber/v2"
)

func HandleCreateCategory(c *fiber.Ctx) error {
	var category models.Category
	if err := c.BodyParser(&category); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid request body",
		})
	}

	if err := category.Create(); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to create category",
		})
	}

	return c.Status(fiber.StatusCreated).JSON(category)
}

func HandleGetAllCategories(c *fiber.Ctx) error {
	categories, err := models.GetAllCategories()
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to get categories",
		})
	}

	return c.JSON(categories)
}

func HandleGetCategory(c *fiber.Ctx) error {
	id := c.Params("id")
	category, err := models.GetCategoryByID(id)
	if err != nil {
		return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
			"error": "category not found",
		})
	}

	return c.JSON(category)
}

func HandleUpdateCategory(c *fiber.Ctx) error {
	id := c.Params("id")
	var category models.Category
	if err := c.BodyParser(&category); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid request body",
		})
	}

	category.CategoryID = id
	if err := category.Update(); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to update category",
		})
	}

	return c.JSON(category)
}

func HandleDeleteCategory(c *fiber.Ctx) error {
	id := c.Params("id")
	if err := models.DeleteCategory(id); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to delete category",
		})
	}

	return c.SendStatus(fiber.StatusNoContent)
}
