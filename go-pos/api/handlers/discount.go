package handlers

import (
	"go-pos/api/models"
	"github.com/gofiber/fiber/v2"
)

func HandleCreateDiscount(c *fiber.Ctx) error {
	var discount models.Discount
	if err := c.BodyParser(&discount); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid request body",
		})
	}

	if err := discount.Create(); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to create discount",
		})
	}

	return c.Status(fiber.StatusCreated).JSON(discount)
}

func HandleGetAllDiscounts(c *fiber.Ctx) error {
	discounts, err := models.GetAllDiscounts()
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to get discounts",
		})
	}

	return c.JSON(discounts)
}

func HandleGetDiscount(c *fiber.Ctx) error {
	id := c.Params("id")
	discount, err := models.GetDiscountByID(id)
	if err != nil {
		return c.Status(fiber.StatusNotFound).JSON(fiber.Map{
			"error": "discount not found",
		})
	}

	return c.JSON(discount)
}

func HandleUpdateDiscount(c *fiber.Ctx) error {
	id := c.Params("id")
	var discount models.Discount
	if err := c.BodyParser(&discount); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
			"error": "invalid request body",
		})
	}

	discount.DiscountID = id
	if err := discount.Update(); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to update discount",
		})
	}

	return c.JSON(discount)
}

func HandleDeleteDiscount(c *fiber.Ctx) error {
	id := c.Params("id")
	if err := models.DeleteDiscount(id); err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
			"error": "failed to delete discount",
		})
	}

	return c.SendStatus(fiber.StatusNoContent)
}
