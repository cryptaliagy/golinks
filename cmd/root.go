/*
Copyright © 2022 NAME HERE <EMAIL ADDRESS>

*/
package cmd

import (
	"errors"
	"log"
	"os"

	"github.com/spf13/cobra"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

type rootConfig struct {
	dbFile string
	db     *gorm.DB
}

type Link struct {
	gorm.Model
	Url string `json:"url"`
	Tag string `json:"tag"`
}

var cliConstants = rootConfig{
	dbFile: "links.db",
	db:     nil,
}

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "golinks",
	Short: "A brief description of your application",
	Long: `A longer description that spans multiple lines and likely contains
examples and usage of using your application. For example:

Cobra is a CLI library for Go that empowers applications.
This application is a tool to generate the needed files
to quickly create a Cobra application.`,
	// Uncomment the following line if your bare application
	// has an action associated with it:
}

// Execute adds all child commands to the root command and sets flags appropriately.
// This is called by main.main(). It only needs to happen once to the rootCmd.
func Execute() {
	rootCmd.ParseFlags(os.Args)
	cliConstants.dbFile = rootCmd.Flag("database").Value.String()

	log.Println("Using database", cliConstants.dbFile)
	cliConstants.db, err = gorm.Open(
		sqlite.Open(cliConstants.dbFile),
		&gorm.Config{},
	)

	if err != nil {
		log.Fatal(err)
	}

	db := cliConstants.db
	db.AutoMigrate(&Link{})

	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

func init() {
	// Here you will define your flags and configuration settings.
	// Cobra supports persistent flags, which, if defined here,
	// will be global for your application.

	// rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "config file (default is $HOME/.golinks.yaml)")

	// Cobra also supports local flags, which will only run
	// when this action is called directly.
	rootCmd.PersistentFlags().
		String("database", cliConstants.dbFile, "The database to use")

}

func addRoute(link string, url string) error {
	var newLink Link
	cliConstants.db.Find(&newLink, "tag = ?", link)
	if newLink.ID == 0 {
		newLink.Tag = link
		newLink.Url = url
		cliConstants.db.Create(&newLink)
	} else {
		return errors.New("Link already exists")
	}

	return nil

}

func updateLink(link string, url string) error {
	var redirect Link
	cliConstants.db.Where("tag = ?", link).First(&redirect)
	if redirect.ID == 0 {
		return errors.New("Link does not exist")
	} else {
		redirect.Url = url
		cliConstants.db.Save(&redirect)
	}

	return nil
}

func removeRoute(link string) error {
	var redirect Link
	cliConstants.db.Where("tag = ?", link).First(&redirect)
	if redirect.ID == 0 {
		return errors.New("Link does not exist")
	} else {
		cliConstants.db.Delete(&redirect)
	}

	return nil
}

func allRoutes() map[string]string {
	links := []Link{}
	cliConstants.db.Find(&links)

	routes := make(map[string]string)
	for _, link := range links {
		routes[link.Tag] = link.Url
	}

	return routes
}

func clearRoutes() error {
	db := cliConstants.db

	links := []Link{}
	db.Find(&links)

	for _, link := range links {
		db.Delete(&link)
	}

	db.Find(&links)

	if len(links) > 0 {
		return errors.New("Failed to clear routes")
	}

	return nil
}

func getRedirect(link string) string {
	var redirect Link
	cliConstants.db.Where("tag = ?", link).First(&redirect)
	return redirect.Url
}
