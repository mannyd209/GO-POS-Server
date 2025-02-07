package config

import (
	"os"
	"path/filepath"
)

type Config struct {
	Port     string
	DBPath   string
	AppEnv   string
}

func LoadConfig() *Config {
	return &Config{
		Port:     getEnv("PORT", "8000"),
		DBPath:   getEnv("DB_PATH", filepath.Join("Database", "pos.db")),
		AppEnv:   getEnv("APP_ENV", "development"),
	}
}

func getEnv(key, fallback string) string {
	if value, exists := os.LookupEnv(key); exists {
		return value
	}
	return fallback
}
