package middleware

import (
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/session"
)

var store *session.Store

func InitStore(s *session.Store) {
	store = s
}

func GetStore() *session.Store {
	return store
}

func Protected() fiber.Handler {
	return func(c *fiber.Ctx) error {
		sess, err := store.Get(c)
		if err != nil {
			return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{
				"error": "invalid session",
			})
		}

		staffID := sess.Get("staff_id")
		if staffID == nil {
			return c.Status(fiber.StatusUnauthorized).JSON(fiber.Map{
				"error": "not authenticated",
			})
		}

		c.Locals("staff_id", staffID)
		c.Locals("is_admin", sess.Get("is_admin"))
		return c.Next()
	}
}

func AdminOnly() fiber.Handler {
	return func(c *fiber.Ctx) error {
		isAdmin := c.Locals("is_admin")
		if isAdmin == nil || !isAdmin.(bool) {
			return c.Status(fiber.StatusForbidden).JSON(fiber.Map{
				"error": "admin access required",
			})
		}
		return c.Next()
	}
}
