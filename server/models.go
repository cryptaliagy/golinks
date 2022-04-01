package server

import (
	"gorm.io/gorm"
)

type Link struct {
	gorm.Model
	Url string `json:"url"`
	Tag string `json:"tag"`
}

type Payload struct {
	Url string `json:"url"`
}

type ServerConfig struct {
	Port     string
	Key      string
	IsDebug  bool
	Database *gorm.DB
}

type DatabaseProvider interface {
	InitDB(string)
	ValidateAuth(string) bool
	GetAllRoutes() map[string]string
	UpdateRoute(string, string) error
	RemoveRoute(string) error
	ClearRoutes() error
	AddRoute(string, string) error
	GetRoute(string) (string, error)
}
