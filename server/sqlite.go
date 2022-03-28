package server

import (
	"errors"
	"log"

	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

type SQLiteServerConfig struct {
	ServerConfig
	DatabaseFile string
}

func (s *SQLiteServerConfig) InitDB(databaseFile string) {
	var err error

	s.DatabaseFile = databaseFile
	s.Database, err = gorm.Open(
		sqlite.Open(databaseFile),
		&gorm.Config{},
	)

	if err != nil {
		log.Fatal(err)
	}

	s.Database.AutoMigrate(&Link{})
}

func (s *SQLiteServerConfig) GetAllRoutes() map[string]string {
	var links []Link

	s.Database.Find(&links)

	routes := make(map[string]string)

	for _, link := range links {
		routes[link.Tag] = link.Url
	}

	return routes
}

func (s *SQLiteServerConfig) UpdateRoute(link string, url string) error {
	var redirect Link
	tx := s.Database.Begin()
	tx.Where("tag = ?", link).First(&redirect)
	if redirect.ID == 0 {
		return errors.New("Link does not exist")
	} else {
		redirect.Url = url
		tx.Save(&redirect)
	}

	tx.Commit()

	return nil
}

func (s *SQLiteServerConfig) RemoveRoute(link string) error {
	var redirect Link

	tx := s.Database.Begin()

	tx.Where("tag = ?", link).First(&redirect)
	if redirect.ID == 0 {
		return errors.New("Link does not exist")
	} else {
		tx.Delete(&redirect)
	}

	tx.Commit()

	return nil
}

func (s *SQLiteServerConfig) ClearRoutes() error {
	s.Database.Where("1 == 1").Delete(&Link{}).Commit()

	var links []Link
	s.Database.Find(&links)

	if len(links) > 0 {
		return errors.New("Failed to clear routes")
	}

	return nil
}

func (s *SQLiteServerConfig) AddRoute(link string, url string) error {
	linkToAdd := Link{
		Url: url,
		Tag: link,
	}

	s.Database.Create(&linkToAdd).Commit()

	return nil
}

func (s *SQLiteServerConfig) GetRoute(link string) (string, error) {
	var redirect Link
	s.Database.Where("tag = ?", link).First(&redirect)
	if redirect.ID == 0 {
		return "", errors.New("Link does not exist")
	}

	log.Printf("Found route %s", redirect.Url)

	return redirect.Url, nil
}

func (s *SQLiteServerConfig) ValidateAuth(key string) bool {
	return s.Key == key
}
