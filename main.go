package main

import (
	"errors"
	"log"
	"net/http"
	"os"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/joho/godotenv"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

type config struct {
	db  *gorm.DB
	key string
}

type Link struct {
	gorm.Model
	Url string `json:"url"`
	Tag string `json:"tag"`
}

var confs config
var err error

func main() {
	godotenv.Load()
	confs.key = os.Getenv("SECRET_KEY")
	confs.db, err = gorm.Open(sqlite.Open("test.db"), &gorm.Config{})

	if err != nil {
		panic(err)
	}

	confs.db.AutoMigrate(&Link{})

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

	r.Run()
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

	if token != confs.key {
		c.IndentedJSON(http.StatusUnauthorized, gin.H{
			"ok":  false,
			"msg": "Invalid token",
		})
		c.Abort()
		return
	}

	c.Next()
}

func addRoute(link string, url string) error {
	var newLink Link
	confs.db.Find(&newLink, "tag = ?", link)
	if newLink.ID == 0 {
		newLink.Tag = link
		newLink.Url = url
		confs.db.Create(&newLink)
	} else {
		return errors.New("Link already exists")
	}

	return nil

}

func updateLink(link string, url string) {
	confs.db.Model(&Link{}).Where("tag = ?", link).Update("url", url)
}

func removeRoute(link string) {
	confs.db.Model(&Link{}).Where("tag = ?", link).Delete(&Link{})
}

func allRoutes() map[string]string {
	links := []Link{}
	confs.db.Find(&links)

	routes := make(map[string]string)
	for _, link := range links {
		routes[link.Tag] = link.Url
	}

	return routes
}

func getRedirect(link string) string {
	var redirect Link
	confs.db.Where("tag = ?", link).First(&redirect)
	return redirect.Url
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
