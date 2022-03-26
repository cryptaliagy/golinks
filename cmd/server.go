/*
Copyright © 2022 NAME HERE <EMAIL ADDRESS>

*/
package cmd

import (
	"log"
	"net/http"
	"os"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/joho/godotenv"
	"github.com/spf13/cobra"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

type serverConfig struct {
	Port    string
	Key     string
	IsDebug bool
}

var config serverConfig
var err error

// serverCmd represents the server command
var serverCmd = &cobra.Command{
	Use:   "server",
	Short: "Starts up the redirect server",
	Long:  `Starts up the GoLinks redirect server.`,
	Run: func(cmd *cobra.Command, args []string) {
		godotenv.Load()
		cmd.Flags().Parse(args)
		config = serverConfig{
			Port:    cmd.Flag("port").Value.String(),
			IsDebug: cmd.Flag("debug").Value.String() == "true",
			Key:     os.Getenv("GOLINKS_SECRET_KEY"),
		}
		cliConstants.db, err = gorm.Open(sqlite.Open(cliConstants.dbFile), &gorm.Config{})
		if err != nil {
			log.Fatal(err)
		}
		db := cliConstants.db

		db.AutoMigrate(&Link{})

		if config.IsDebug {
			gin.SetMode(gin.DebugMode)
		} else {
			gin.SetMode(gin.ReleaseMode)
		}

		r := gin.Default()

		r.SetTrustedProxies(nil)
		r.GET("/heartbeat", heartbeat)
		r.GET("/:link", doRedirect)
		r.GET("/which/:link", getLink)
		r.GET("/all", getAllRoutes)

		private := r.Group("/route")
		private.Use(Authentication)
		{
			private.POST("/:link", addRouteToMap)
			private.DELETE("/:link", deleteRoute)
			private.PUT("/:link", updateRoute)
		}

		r.Run(":" + config.Port)
	},
}

func init() {
	rootCmd.AddCommand(serverCmd)

	serverCmd.
		Flags().
		StringP("port", "p", "8080", "port to listen on")

	serverCmd.
		Flags().
		BoolP("debug", "d", false, "debug mode")

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// serverCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// serverCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}

func Authentication(c *gin.Context) {
	auth_header := c.Request.Header.Get("Authorization")

	if auth_header == "" {
		c.IndentedJSON(http.StatusUnauthorized, gin.H{
			"ok":  false,
			"msg": "No Authorization header",
		})
		c.Abort()
		return
	}

	token := strings.TrimPrefix(auth_header, "Bearer ")

	if token != config.Key {
		c.IndentedJSON(http.StatusUnauthorized, gin.H{
			"ok":  false,
			"msg": "Invalid token",
		})
		c.Abort()
		return
	}

	c.Next()
}

func addRouteToMap(c *gin.Context) {
	link := c.Param("link")
	var newUrl Link
	err := c.BindJSON(&newUrl)
	if err != nil {
		log.Println(err)

	}

	log.Println(newUrl.Url)

	if strings.HasPrefix(newUrl.Url, "http") {
		err = addRoute(link, newUrl.Url)
		if err != nil {
			c.IndentedJSON(http.StatusBadRequest, gin.H{
				"ok":  false,
				"msg": err.Error(),
			})
			return
		}
		c.IndentedJSON(http.StatusOK, gin.H{
			"ok":  true,
			"msg": "Link added",
		})
	} else {
		c.IndentedJSON(http.StatusBadRequest, gin.H{
			"ok":  false,
			"msg": "Invalid URL",
		})
	}
}

func getAllRoutes(c *gin.Context) {
	routes := allRoutes()
	c.IndentedJSON(http.StatusOK, gin.H{
		"ok":     true,
		"msg":    "All routes",
		"routes": routes,
		"count":  len(routes),
	})
}

func deleteRoute(c *gin.Context) {
	link := c.Param("link")
	removeRoute(link)
	c.IndentedJSON(http.StatusOK, gin.H{
		"ok":  true,
		"msg": "Link deleted",
	})
}

func updateRoute(c *gin.Context) {
	link := c.Param("link")
	var newUrl Link
	c.BindJSON(&newUrl)
	if strings.HasPrefix(newUrl.Url, "http") {
		updateLink(link, newUrl.Url)
		c.IndentedJSON(http.StatusOK, gin.H{
			"ok":  true,
			"msg": "Link added",
		})
	} else {
		c.IndentedJSON(http.StatusBadRequest, gin.H{
			"ok":  false,
			"msg": "Invalid URL",
		})
	}
}

func heartbeat(c *gin.Context) {
	c.IndentedJSON(http.StatusOK, gin.H{
		"ok":  true,
		"msg": "Service is up and running",
	})
}

func getLink(c *gin.Context) {
	link := c.Param("link")
	url := getRedirect(link)

	if url == "" {
		c.IndentedJSON(http.StatusNotFound, gin.H{
			"ok":  false,
			"msg": "Link not found",
		})
		return
	}
	c.IndentedJSON(http.StatusOK, gin.H{
		"ok":  true,
		"url": url,
	})
}

func doRedirect(c *gin.Context) {
	link := c.Param("link")
	url := getRedirect(link)
	if url == "" {
		c.IndentedJSON(http.StatusNotFound, gin.H{
			"ok":  false,
			"msg": "Link not found",
		})
		return
	}
	c.Redirect(http.StatusTemporaryRedirect, url)
}
